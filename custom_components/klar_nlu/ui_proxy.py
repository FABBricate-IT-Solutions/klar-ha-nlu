"""Authenticated reverse proxy from Home Assistant to the configured Klar engine."""

from __future__ import annotations

import hmac
import logging
import secrets
import time
from hashlib import sha256
from typing import Any
from urllib.parse import urljoin, urlparse, urlunparse

from aiohttp import ClientError, ClientTimeout, web
from homeassistant.core import HomeAssistant
from homeassistant.helpers.aiohttp_client import async_get_clientsession

try:
    from homeassistant.components.http import HomeAssistantView
except ImportError:
    from homeassistant.components.http.view import HomeAssistantView

from .const import (
    DOMAIN,
    TOKEN_HEADER,
    addon_sidebar_path,
)

_LOGGER = logging.getLogger(__name__)

UI_PREFIX = "/api/klar_nlu/ui"
SESSION_PATH = "/api/klar_nlu/session"
COOKIE_NAME = "klar_nlu_ui"
COOKIE_PATH = "/api/klar_nlu"
COOKIE_TTL_S = 86400
_STORE_KEYS = frozenset({"panel", "ui_proxy", "ui_cookie_secret", "operator_panel"})
_STRIP_REQ = frozenset(
    {
        "authorization",
        "connection",
        "content-length",
        "cookie",
        "host",
        "keep-alive",
        "proxy-authenticate",
        "proxy-authorization",
        "te",
        "trailer",
        "transfer-encoding",
        "upgrade",
    }
)
_STRIP_RESP = frozenset(
    {
        "connection",
        "content-encoding",
        "content-length",
        "keep-alive",
        "set-cookie",
        "transfer-encoding",
    }
)


def configured_engine(hass: HomeAssistant) -> tuple[str, str | None]:
    """Return the integration's engine URL and token. Never reads a request URL."""
    stored = hass.data.get(DOMAIN) or {}
    for key, payload in stored.items():
        if key in _STORE_KEYS or not isinstance(payload, dict):
            continue
        url = str(payload.get("url") or "").strip().rstrip("/")
        if not url:
            continue
        token = payload.get("token")
        return url, str(token) if token else None
    return "", None


def join_engine_url(base: str, path: str, query: str = "") -> str:
    """Map a proxy suffix onto the configured engine origin. Reject off-origin targets."""
    origin = str(base or "").strip().rstrip("/")
    if not origin:
        raise ValueError("engine url missing")
    raw = (path or "").replace("\\", "/")
    if raw.startswith("//") or "://" in raw:
        raise ValueError("refusing absolute proxy path")
    if "\x00" in raw:
        raise ValueError("invalid path")
    target = urljoin(origin + "/", raw.lstrip("/"))
    if query:
        parsed = urlparse(target)
        target = urlunparse(parsed._replace(query=query))
    if not same_engine_origin(origin, target):
        raise ValueError("refusing off-origin proxy target")
    return target


def same_engine_origin(base: str, target: str) -> bool:
    left, right = urlparse(base), urlparse(target)
    return (
        left.scheme == right.scheme
        and (left.hostname or "").lower() == (right.hostname or "").lower()
        and _origin_port(left) == _origin_port(right)
    )


def _origin_port(parsed: Any) -> int:
    if parsed.port:
        return int(parsed.port)
    return 443 if parsed.scheme == "https" else 80


def mint_ui_cookie(secret: bytes, user_id: str, now: float, ttl: int = COOKIE_TTL_S) -> str:
    uid = str(user_id or "user").replace("|", "_")[:64]
    exp = int(now) + ttl
    sig = hmac.new(secret, f"{uid}|{exp}".encode(), sha256).hexdigest()
    return f"{uid}|{exp}|{sig}"


def valid_ui_cookie(secret: bytes, value: str, now: float) -> bool:
    parts = str(value or "").split("|")
    if len(parts) != 3:
        return False
    uid, exp_s, sig = parts
    try:
        exp = int(exp_s)
    except ValueError:
        return False
    if exp < int(now) or not uid or not sig:
        return False
    expected = hmac.new(secret, f"{uid}|{exp}".encode(), sha256).hexdigest()
    return hmac.compare_digest(expected, sig)


def cookie_secret(hass: HomeAssistant) -> bytes:
    store = hass.data.setdefault(DOMAIN, {})
    secret = store.get("ui_cookie_secret")
    if not isinstance(secret, bytes):
        secret = secrets.token_bytes(32)
        store["ui_cookie_secret"] = secret
    return secret


def ha_request_authenticated(request: web.Request) -> bool:
    if request.get("hass_user") is not None:
        return True
    return bool(request.get("ha_authenticated"))


def request_allowed(request: web.Request, secret: bytes, now: float | None = None) -> bool:
    if ha_request_authenticated(request):
        return True
    cookie = request.cookies.get(COOKIE_NAME) if request.cookies else None
    return valid_ui_cookie(secret, str(cookie or ""), now if now is not None else time.time())


def addon_ingress_active(hass: HomeAssistant) -> bool:
    """True when Supervisor already put this Klar engine on the sidebar."""
    panels = hass.data.get("frontend_panels") or {}
    wanted = {addon_sidebar_path(url) for url in _configured_urls(hass)}
    wanted.discard(None)
    for path in wanted:
        panel = panels.get(path) if isinstance(panels, dict) else None
        if panel is not None and _panel_component(panel) == "hassio":
            return True
    return False


def _configured_urls(hass: HomeAssistant) -> list[str]:
    urls: list[str] = []
    for key, payload in (hass.data.get(DOMAIN) or {}).items():
        if key in _STORE_KEYS or not isinstance(payload, dict):
            continue
        url = str(payload.get("url") or "").strip()
        if url:
            urls.append(url)
    return urls


def _panel_component(panel: object) -> str:
    name = getattr(panel, "component_name", None)
    if name:
        return str(name)
    if isinstance(panel, dict):
        return str(panel.get("component_name") or "")
    return ""


def _user_id(request: web.Request) -> str:
    user = request.get("hass_user")
    return str(getattr(user, "id", None) or "user")


def _set_ui_cookie(response: web.StreamResponse, request: web.Request, hass: HomeAssistant) -> None:
    response.set_cookie(
        COOKIE_NAME,
        mint_ui_cookie(cookie_secret(hass), _user_id(request), time.time()),
        max_age=COOKIE_TTL_S,
        httponly=True,
        samesite="Lax",
        path=COOKIE_PATH,
        secure=request.secure or request.headers.get("X-Forwarded-Proto") == "https",
    )


def _forward_headers(incoming: Any, token: str | None) -> dict[str, str]:
    headers: dict[str, str] = {}
    for key, value in incoming.items():
        low = str(key).lower()
        if low in _STRIP_REQ or low.startswith("x-forwarded-"):
            continue
        headers[str(key)] = str(value)
    if token:
        headers[TOKEN_HEADER] = token
    return headers


def _response_headers(incoming: Any, engine_base: str) -> dict[str, str]:
    headers: dict[str, str] = {}
    for key, value in incoming.items():
        if str(key).lower() in _STRIP_RESP:
            continue
        if str(key).lower() == "location":
            rewritten = _rewrite_location(engine_base, str(value))
            if rewritten:
                headers["Location"] = rewritten
            continue
        headers[str(key)] = str(value)
    return headers


def _rewrite_location(engine_base: str, location: str) -> str | None:
    target = urljoin(engine_base.rstrip("/") + "/", location)
    if not same_engine_origin(engine_base, target):
        return None
    parsed = urlparse(target)
    suffix = parsed.path or "/"
    if parsed.query:
        suffix = f"{suffix}?{parsed.query}"
    return f"{UI_PREFIX}{suffix}"


async def async_setup_ui_proxy(hass: HomeAssistant) -> None:
    if (hass.data.get(DOMAIN) or {}).get("ui_proxy"):
        return
    hass.data.setdefault(DOMAIN, {})["ui_proxy"] = True
    hass.http.register_view(KlarUiSessionView())
    hass.http.register_view(KlarUiProxyView())


class KlarUiSessionView(HomeAssistantView):
    """Mint a same-origin cookie so the operator iframe can load assets and /api."""

    url = SESSION_PATH
    name = "api:klar_nlu:session"
    requires_auth = True

    async def post(self, request: web.Request) -> web.Response:
        response = web.Response(status=204)
        _set_ui_cookie(response, request, request.app["hass"])
        return response


class KlarUiProxyView(HomeAssistantView):
    """Proxy UI + engine API under one prefix. Auth via HA user or session cookie."""

    url = UI_PREFIX
    extra_urls = [f"{UI_PREFIX}/{{path:.*}}"]
    name = "api:klar_nlu:ui"
    requires_auth = False
    requires_ha_auth = True

    async def get(self, request: web.Request, path: str = "") -> web.StreamResponse:
        return await self._handle(request, path)

    async def post(self, request: web.Request, path: str = "") -> web.StreamResponse:
        return await self._handle(request, path)

    async def put(self, request: web.Request, path: str = "") -> web.StreamResponse:
        return await self._handle(request, path)

    async def delete(self, request: web.Request, path: str = "") -> web.StreamResponse:
        return await self._handle(request, path)

    async def patch(self, request: web.Request, path: str = "") -> web.StreamResponse:
        return await self._handle(request, path)

    async def head(self, request: web.Request, path: str = "") -> web.StreamResponse:
        return await self._handle(request, path)

    async def _handle(self, request: web.Request, path: str) -> web.StreamResponse:
        hass: HomeAssistant = request.app["hass"]
        secret = cookie_secret(hass)
        if not request_allowed(request, secret):
            return web.Response(status=401, text="Authentication required")
        base, token = configured_engine(hass)
        if not base:
            return web.Response(status=503, text="Klar engine is not configured")
        try:
            target = join_engine_url(base, path, request.query_string)
        except ValueError:
            return web.Response(status=400, text="Invalid path")
        body = await request.read() if request.can_read_body else None
        session = async_get_clientsession(hass)
        try:
            async with session.request(
                request.method,
                target,
                headers=_forward_headers(request.headers, token),
                data=body,
                timeout=ClientTimeout(total=300, sock_connect=10),
                allow_redirects=False,
            ) as resp:
                headers = _response_headers(resp.headers, base)
                if request.method == "HEAD":
                    out: web.StreamResponse = web.Response(status=resp.status, headers=headers)
                    if ha_request_authenticated(request):
                        _set_ui_cookie(out, request, hass)
                    return out
                streamed = web.StreamResponse(status=resp.status, headers=headers)
                if ha_request_authenticated(request):
                    _set_ui_cookie(streamed, request, hass)
                await streamed.prepare(request)
                async for chunk in resp.content.iter_any():
                    await streamed.write(chunk)
                await streamed.write_eof()
                return streamed
        except (ClientError, TimeoutError, OSError) as err:
            _LOGGER.warning("Klar UI proxy failed: %s", err)
            return web.Response(status=502, text="Klar engine is unreachable")
