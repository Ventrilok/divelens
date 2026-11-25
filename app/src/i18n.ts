// How the interface speaks: the language (the system's unless chosen in
// Settings, changed at once, spec F2), plurals and placeholders. The texts
// are in i18n/, one file per language; English is the reference.

import { en, type Key } from "./i18n/en";
import { de } from "./i18n/de";
import { fr } from "./i18n/fr";

export type { Key };
export type Lang = "en" | "fr" | "de";

const dictionaries: Record<Lang, Record<Key, string>> = { en, fr, de };

/** The system's language, when DiveLens speaks it; English otherwise. */
export function systemLang(): Lang {
  const system = navigator.language.toLowerCase();
  if (system.startsWith("fr")) return "fr";
  if (system.startsWith("de")) return "de";
  return "en";
}

const locales: Record<Lang, string> = { en: "en-GB", fr: "fr-FR", de: "de-DE" };
const localeOf = (l: Lang) => locales[l];

export let lang: Lang = systemLang();
export let locale = localeOf(lang);

/** Switches the interface language ("system": follow the system). */
export function setLanguage(choice: Lang | "system"): void {
  lang = choice === "system" ? systemLang() : choice;
  locale = localeOf(lang);
}

/** An Intl formatter that follows the language: rebuilt when it changes. */
export function localized<T extends object>(make: (locale: string) => T): T {
  let madeFor = "";
  let value: T;
  const current = () => {
    if (madeFor !== locale) {
      value = make(locale);
      madeFor = locale;
    }
    return value;
  };
  return new Proxy({} as T, {
    get(_, key) {
      const target = current();
      const v = Reflect.get(target, key);
      return typeof v === "function" ? v.bind(target) : v;
    },
  });
}

const plurals = localized((l) => new Intl.PluralRules(l));

type Params = Record<string, string | number>;

/** `text` with its `{name}` placeholders filled from `params`. */
export function fill(text: string, params?: Params): string {
  if (!params) return text;
  return text.replace(/\{(\w+)\}/g, (_, k: string) => String(params[k] ?? `{${k}}`));
}

/** Translated string. */
export function t(key: Key, params?: Params): string {
  return fill(dictionaries[lang][key], params);
}

type PluralBase<K extends string> = K extends `${infer B}_one` ? B : never;

/** Translated string with plural agreement on `n` (keys `<base>_one` / `<base>_other`). */
export function tn(base: PluralBase<Key>, n: number, params?: Params): string {
  const form = plurals.select(n) === "one" ? "one" : "other";
  return t(`${base}_${form}` as Key, { n: formatInt(n), ...params });
}

const integers = localized((l) => new Intl.NumberFormat(l));

export function formatInt(n: number): string {
  return integers.format(n);
}
