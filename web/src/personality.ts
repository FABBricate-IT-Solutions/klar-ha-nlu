import type { Messages } from "./i18n";
import { PERSONALITIES, type PersonalityId } from "./generated/contract";

export { PERSONALITIES, type PersonalityId };

export function isPersonality(value: string): value is PersonalityId {
  return (PERSONALITIES as readonly string[]).includes(value);
}

export function personalityLabel(t: Messages, id: PersonalityId, customName?: string): string {
  switch (id) {
    case "default":
      return t.personalityDefault;
    case "butler":
      return t.personalityButler;
    case "locker":
      return t.personalityLocker;
    case "fuersorglich":
      return t.personalityFuersorglich;
    case "party":
      return t.personalityParty;
    case "grantig":
      return t.personalityGrantig;
    case "sarkastisch":
      return t.personalitySarkastisch;
    case "pirat":
      return t.personalityPirat;
    case "hippie":
      return t.personalityHippie;
    case "gollum":
      return t.personalityGollum;
    case "jarvis":
      return t.personalityJarvis;
    case "custom":
      return customName?.trim() || t.personalityCustom;
    default: {
      const _never: never = id;
      return _never;
    }
  }
}
