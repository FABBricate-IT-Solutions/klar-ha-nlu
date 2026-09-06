import type { VoiceTraits } from "./types";

export const DEFAULT_TRAITS: VoiceTraits = {
  warmth: 5,
  humor: 4,
  sarcasm: 2,
  formality: 5,
  verbosity: 4,
  energy: 5,
};

export const TRAIT_KEYS = ["warmth", "humor", "sarcasm", "formality", "verbosity", "energy"] as const;

export type TraitKey = (typeof TRAIT_KEYS)[number];

export function readTraits(raw?: VoiceTraits): VoiceTraits {
  return {
    warmth: clampTrait(raw?.warmth, DEFAULT_TRAITS.warmth),
    humor: clampTrait(raw?.humor, DEFAULT_TRAITS.humor),
    sarcasm: clampTrait(raw?.sarcasm, DEFAULT_TRAITS.sarcasm),
    formality: clampTrait(raw?.formality, DEFAULT_TRAITS.formality),
    verbosity: clampTrait(raw?.verbosity, DEFAULT_TRAITS.verbosity),
    energy: clampTrait(raw?.energy, DEFAULT_TRAITS.energy),
  };
}

function clampTrait(value: number | undefined, fallback: number): number {
  if (typeof value !== "number" || Number.isNaN(value)) return fallback;
  return Math.min(10, Math.max(0, Math.round(value)));
}
