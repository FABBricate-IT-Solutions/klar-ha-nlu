"""Tests: resolve artist before playback and stop a wrong match before retry."""
import ast
import asyncio
import difflib
import re
import unittest
from pathlib import Path

SOURCE = Path(__file__).resolve().parents[1] / "dispatch_media.py"
TREE = ast.parse(SOURCE.read_text())
NAMESPACE = {
    "difflib": difflib,
    "re": re,
    "asyncio": asyncio,
    "Any": object,
    "_PUNCT": re.compile(r"[^\w\s]+", re.UNICODE),
    "_ROOM_FILLER": {"in", "on", "at", "the", "a", "an", "display", "speaker",
                     "room", "bedroom", "kitchen", "living", "dining"},
}
WANTED = {"clean_media_query", "_token_similarity", "choose_artist",
          "canonical_play_data", "stop_media_playback"}
for node in TREE.body:
    if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)) and node.name in WANTED:
        exec(compile(ast.Module(body=[node], type_ignores=[]), str(SOURCE), "exec"), NAMESPACE)
canonical_play_data = NAMESPACE["canonical_play_data"]
stop_media_playback = NAMESPACE["stop_media_playback"]

LIBRARY = ["Mandy Moore", "Dido", "Katelyn Tarver", "Calvin Harris", "Queen"]


class FakeServices:
    def __init__(self):
        self.calls = []

    async def async_call(self, domain, service, data, *, blocking, target):
        self.calls.append((domain, service, data, blocking, target))


class FakeHass:
    def __init__(self):
        self.services = FakeServices()


class MassPlayGuardTests(unittest.TestCase):
    def test_canonical_artist_is_used_before_first_play(self):
        data = {"media_id": "caitlin tarver", "media_type": "artist", "enqueue": "replace"}
        result = canonical_play_data(data, LIBRARY)
        self.assertEqual(result["media_id"], "Katelyn Tarver")
        self.assertEqual(result["enqueue"], "replace")

    def test_non_artist_play_data_is_untouched(self):
        data = {"media_id": "some track", "media_type": "track"}
        self.assertEqual(canonical_play_data(data, LIBRARY), data)

    def test_wrong_match_stops_playback_before_retry(self):
        hass = FakeHass()
        asyncio.run(stop_media_playback(hass, "media_player.bedroom_display_2"))
        self.assertEqual(hass.services.calls, [
            ("media_player", "media_stop", {}, True,
             {"entity_id": "media_player.bedroom_display_2"})
        ])


if __name__ == "__main__":
    unittest.main()
