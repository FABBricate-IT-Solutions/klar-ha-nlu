"""Phrase pools for the generated held-out cases. Written for Klar, not copied from any suite.

`{n}` is an entity name, `{a}` an area name, `{p}` a brightness percentage, `{d}` degrees.
"""

from __future__ import annotations

LANGS: dict[str, dict] = {
    "en": {
        "onoff": {
            "light": {
                "on": [
                    "I'd like the {n} on",
                    "{n} on please",
                    "Would you turn on the {n}",
                    "Could you switch on the {n} for me",
                    "Please turn on the {n}",
                    "Turn the {n} on",
                    "Can you turn on the {n}",
                    "Switch the {n} on",
                ],
                "off": [
                    "I'd like the {n} off",
                    "{n} off please",
                    "Would you turn off the {n}",
                    "Could you switch off the {n} for me",
                    "Kill the {n}",
                    "Shut off the {n}",
                    "Turn the {n} off",
                    "Can you turn off the {n}",
                ],
            },
            "switch": {
                "on": [
                    "Start the {n}",
                    "{n} on please",
                    "Can you turn the {n} on",
                    "Would you switch on the {n}",
                    "Power up the {n}",
                    "I'd like the {n} on",
                    "Could you start the {n} for me",
                    "Switch the {n} on",
                ],
                "off": [
                    "Stop the {n}",
                    "Shut off the {n}",
                    "{n} off please",
                    "Can you turn the {n} off",
                    "Would you switch off the {n}",
                    "Power down the {n}",
                    "I'd like the {n} off",
                    "Switch the {n} off",
                ],
            },
            "fan": {
                "on": [
                    "Start the {n}",
                    "{n} on please",
                    "Can you turn the {n} on",
                    "Would you switch on the {n}",
                    "Turn the {n} on",
                    "I'd like the {n} on",
                    "Could you start the {n} for me",
                    "Switch the {n} on",
                ],
                "off": [
                    "Stop the {n}",
                    "Shut off the {n}",
                    "{n} off please",
                    "Can you turn the {n} off",
                    "Would you switch off the {n}",
                    "Turn the {n} off",
                    "I'd like the {n} off",
                    "Switch the {n} off",
                ],
            },
        },
        "brightness": ["Dim the {n} to {p} percent", "Set the {n} to {p}%", "Set the {n} brightness to {p} percent"],
        "query": ["Is the {n} on", "Is the {n} off", "Is the {n} currently on"],
        "query_lock": ["Is the {n} locked", "Is the {n} unlocked"],
        "climate": [
            "Set the {n} to {d} degrees",
            "Make the {n} {d} degrees",
            "Put the {n} at {d}",
            "Change the {n} to {d}",
        ],
        "cover_open": ["Open the {n}", "Raise the {n}", "Roll up the {n}", "Pull up the {n}"],
        "room": {
            "on": ["Turn on the lights in the {a}", "Switch on all the lights in the {a}", "Put the {a} lights on"],
            "off": ["Turn off the lights in the {a}", "Switch off all the lights in the {a}", "Put the {a} lights off"],
        },
        "multi": ["{a} and {b}", "{a} and then {b}", "{a}, {b}"],
        "multi_second": {"on": "turn on the {n}", "off": "turn off the {n}"},
        "typo": {
            "on": ["Turn on the {n}", "Switch on the {n}", "Please turn on the {n}"],
            "off": ["Turn off the {n}", "Switch off the {n}", "Shut off the {n}"],
        },
        "confirm": {
            "lock": ["Lock the {n}", "Please lock the {n}"],
            "unlock": ["Unlock the {n}", "Please unlock the {n}"],
            "open": ["Open the {n}", "Please open the {n}"],
        },
    },
    "de": {
        "onoff": {
            "light": {
                "on": [
                    "Mach {n} an",
                    "Schalte {n} ein",
                    "{n} an bitte",
                    "Kannst du {n} anmachen",
                    "Bitte {n} einschalten",
                    "{n} einschalten",
                    "Würdest du {n} anschalten",
                    "Ich möchte {n} einschalten",
                ],
                "off": [
                    "Mach {n} aus",
                    "Schalte {n} aus",
                    "{n} aus bitte",
                    "Kannst du {n} ausmachen",
                    "Bitte {n} ausschalten",
                    "{n} ausschalten",
                    "Würdest du {n} ausschalten",
                    "Ich möchte {n} ausschalten",
                ],
            },
            "switch": {
                "on": [
                    "Starte {n}",
                    "Mach {n} an",
                    "{n} an bitte",
                    "Kannst du {n} anmachen",
                    "Bitte {n} einschalten",
                    "{n} einschalten",
                    "Würdest du {n} anschalten",
                    "Schalte {n} ein",
                ],
                "off": [
                    "Stoppe {n}",
                    "Mach {n} aus",
                    "{n} aus bitte",
                    "Kannst du {n} ausmachen",
                    "Bitte {n} ausschalten",
                    "{n} ausschalten",
                    "Würdest du {n} ausschalten",
                    "Schalte {n} aus",
                ],
            },
            "fan": {
                "on": [
                    "Starte {n}",
                    "Mach {n} an",
                    "{n} an bitte",
                    "Kannst du {n} anmachen",
                    "Bitte {n} einschalten",
                    "{n} einschalten",
                    "Würdest du {n} anschalten",
                    "Schalte {n} ein",
                ],
                "off": [
                    "Stoppe {n}",
                    "Mach {n} aus",
                    "{n} aus bitte",
                    "Kannst du {n} ausmachen",
                    "Bitte {n} ausschalten",
                    "{n} ausschalten",
                    "Würdest du {n} ausschalten",
                    "Schalte {n} aus",
                ],
            },
        },
        "brightness": ["Dimme {n} auf {p} Prozent", "Stelle {n} auf {p} Prozent", "{n} auf {p} Prozent"],
        "query": ["Ist {n} eingeschaltet", "Ist {n} ausgeschaltet", "Ist {n} gerade an"],
        "query_lock": ["Ist {n} abgeschlossen", "Ist {n} zugesperrt"],
        "climate": [
            "Stelle {n} auf {d} Grad",
            "Setze {n} auf {d} Grad",
            "{n} auf {d} Grad",
            "Ändere {n} auf {d} Grad",
        ],
        "cover_open": ["Öffne {n}", "Fahre {n} hoch", "{n} öffnen", "Mach {n} auf"],
        "room": {
            "on": ["Schalte die Lichter im Raum {a} ein", "Mach alle Lichter in {a} an", "Lichter {a} an"],
            "off": ["Schalte die Lichter im Raum {a} aus", "Mach alle Lichter in {a} aus", "Lichter {a} aus"],
        },
        "multi": ["{a} und {b}", "{a} und danach {b}", "{a}, {b}"],
        "multi_second": {"on": "mach {n} an", "off": "mach {n} aus"},
        "typo": {
            "on": ["Mach {n} an", "Schalte {n} ein", "Bitte {n} anmachen"],
            "off": ["Mach {n} aus", "Schalte {n} aus", "Bitte {n} ausmachen"],
        },
        "confirm": {
            "lock": ["Schließe {n} ab", "Bitte {n} abschließen"],
            "unlock": ["Schließe {n} auf", "Bitte {n} aufschließen"],
            "open": ["Öffne {n}", "Bitte {n} öffnen"],
        },
    },
}
