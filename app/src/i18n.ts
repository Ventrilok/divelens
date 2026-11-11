// Interface strings, English and French. Language follows the system for now;
// the Settings screen will let the user override it.

export type Lang = "en" | "fr";

const en = {
  "nav.dives": "Dives",
  "nav.collections": "Create Collections",
  "nav.tag": "Tag Selection",
  "nav.settings": "Settings",
  "nav.soon": "Coming soon",
  "lr.disconnected": "Lightroom not connected",
  "source.macDive": "MacDive",
  "source.dives_one": "{n} dive",
  "source.dives_other": "{n} dives",

  "dives.title": "Dives",
  "dives.withProfile": "{n} with dive profile",
  "dives.loading": "Loading dives…",
  "dives.shown": "{shown} of {total}",
  "sort.label": "Sort",
  "sort.recent": "Newest first",
  "sort.oldest": "Oldest first",
  "sort.depth": "Max depth",

  "filter.search": "No., site, location, country…",
  "filter.searchLabel": "Search dives",
  "filter.period": "Period",
  "filter.from": "From",
  "filter.to": "To",
  "filter.country": "Country",
  "filter.countries": "{n} countries",
  "filter.noCountry": "No country",
  "filter.depth": "Depth",
  "filter.depthMin": "Min",
  "filter.depthMax": "Max",
  "filter.rating": "Rating",
  "filter.ratingAny": "Any",
  "filter.ratingMin": "{n}★ and more",
  "filter.withProfile": "With profile",
  "filter.clear": "Clear",
  "filter.clearAll": "Clear filters",
  "filter.remove": "Remove filter {name}",
  "filter.none": "No dive matches the filters.",

  "col.number": "No.",
  "col.date": "Date",
  "col.site": "Site",
  "col.country": "Country",
  "col.duration": "Duration",
  "col.depth": "Max depth",
  "col.temp": "Temp.",
  "col.rating": "Rating",
  "col.profile": "Profile",
  "list.selectAll": "Select all shown dives",
  "list.select": "Select dive {n}",
  "list.hasProfile": "Profile available",
  "list.noProfile": "No profile",

  "detail.label": "Dive no. {n}",
  "detail.labelNoNumber": "Dive",
  "detail.unknownSite": "Unknown site",
  "detail.duration": "Duration",
  "detail.depth": "Max depth",
  "detail.temp": "Temperature",
  "detail.rating": "Rating",
  "detail.gps": "Site GPS",
  "detail.profileOk": "Profile available · depth per photo possible",
  "detail.noProfile": "No profile: dive entered by hand",
  "detail.noProfileHint": "Photos will get the dive data and the site. Depth, temperature and time will stay empty.",
  "detail.profileLoading": "Loading profile…",
  "detail.profileError": "The dive profile could not be read.",
  "detail.chart": "Dive profile: {depth} max, {duration}",
  "detail.empty": "Choose a dive to see its details.",
  "rating.stars": "{n} stars",
  "rating.none": "Not rated",

  "sel.count_one": "{n} dive selected",
  "sel.count_other": "{n} dives selected",
  "sel.hidden_one": "including {n} hidden by the filters",
  "sel.hidden_other": "including {n} hidden by the filters",
  "sel.clear": "Deselect all",
  "sel.create": "Create Collections",

  "err.notFound.title": "MacDive database not found",
  "err.notFound.text": "There is no database at this location. It may have been moved, for example to iCloud.",
  "err.notADiveLog.title": "This is not a MacDive database",
  "err.notADiveLog.text": "Choose the MacDive.sqlite file.",
  "err.database.title": "The MacDive database could not be read",
  "err.database.text": "If it is stored in iCloud, check that it is downloaded on this Mac.",
  "err.locate": "Locate the database…",
  "err.retry": "Try again",
  "err.dialogFilter": "MacDive database",
} as const;

export type Key = keyof typeof en;

const fr: Record<Key, string> = {
  "nav.dives": "Plongées",
  "nav.collections": "Créer les collections",
  "nav.tag": "Tagger la sélection",
  "nav.settings": "Réglages",
  "nav.soon": "Bientôt disponible",
  "lr.disconnected": "Lightroom déconnecté",
  "source.macDive": "MacDive",
  "source.dives_one": "{n} plongée",
  "source.dives_other": "{n} plongées",

  "dives.title": "Plongées",
  "dives.withProfile": "{n} avec courbe de plongée",
  "dives.loading": "Chargement des plongées…",
  "dives.shown": "{shown} sur {total}",
  "sort.label": "Trier",
  "sort.recent": "Plus récentes d'abord",
  "sort.oldest": "Plus anciennes d'abord",
  "sort.depth": "Profondeur max",

  "filter.search": "N°, site, lieu, pays…",
  "filter.searchLabel": "Rechercher une plongée",
  "filter.period": "Période",
  "filter.from": "Du",
  "filter.to": "Au",
  "filter.country": "Pays",
  "filter.countries": "{n} pays",
  "filter.noCountry": "Sans pays",
  "filter.depth": "Profondeur",
  "filter.depthMin": "Min",
  "filter.depthMax": "Max",
  "filter.rating": "Note",
  "filter.ratingAny": "Toutes",
  "filter.ratingMin": "{n}★ et plus",
  "filter.withProfile": "Avec courbe",
  "filter.clear": "Effacer",
  "filter.clearAll": "Effacer les filtres",
  "filter.remove": "Retirer le filtre {name}",
  "filter.none": "Aucune plongée ne correspond aux filtres.",

  "col.number": "N°",
  "col.date": "Date",
  "col.site": "Site",
  "col.country": "Pays",
  "col.duration": "Durée",
  "col.depth": "Prof. max",
  "col.temp": "Temp.",
  "col.rating": "Note",
  "col.profile": "Courbe",
  "list.selectAll": "Sélectionner toutes les plongées affichées",
  "list.select": "Sélectionner la plongée {n}",
  "list.hasProfile": "Courbe disponible",
  "list.noProfile": "Pas de courbe",

  "detail.label": "Plongée n° {n}",
  "detail.labelNoNumber": "Plongée",
  "detail.unknownSite": "Site inconnu",
  "detail.duration": "Durée",
  "detail.depth": "Prof. max",
  "detail.temp": "Température",
  "detail.rating": "Note",
  "detail.gps": "GPS du site",
  "detail.profileOk": "Courbe disponible · profondeur par photo possible",
  "detail.noProfile": "Pas de courbe : plongée saisie à la main",
  "detail.noProfileHint": "Les photos recevront les infos de plongée et le lieu. Profondeur, température et temps resteront vides.",
  "detail.profileLoading": "Chargement de la courbe…",
  "detail.profileError": "La courbe de cette plongée n'a pas pu être lue.",
  "detail.chart": "Courbe de plongée : {depth} max, {duration}",
  "detail.empty": "Choisissez une plongée pour voir son détail.",
  "rating.stars": "{n} étoiles",
  "rating.none": "Pas de note",

  "sel.count_one": "{n} plongée sélectionnée",
  "sel.count_other": "{n} plongées sélectionnées",
  "sel.hidden_one": "dont {n} masquée par les filtres",
  "sel.hidden_other": "dont {n} masquées par les filtres",
  "sel.clear": "Tout désélectionner",
  "sel.create": "Créer les collections",

  "err.notFound.title": "Base MacDive introuvable",
  "err.notFound.text": "Aucune base à l'emplacement indiqué. Elle a peut-être été déplacée, par exemple dans iCloud.",
  "err.notADiveLog.title": "Ce fichier n'est pas une base MacDive",
  "err.notADiveLog.text": "Choisissez le fichier MacDive.sqlite.",
  "err.database.title": "Impossible de lire la base MacDive",
  "err.database.text": "Si elle est dans iCloud, vérifiez qu'elle est bien téléchargée sur ce Mac.",
  "err.locate": "Localiser la base…",
  "err.retry": "Réessayer",
  "err.dialogFilter": "Base MacDive",
};

const dictionaries: Record<Lang, Record<Key, string>> = { en, fr };

export const lang: Lang = navigator.language.toLowerCase().startsWith("fr") ? "fr" : "en";
export const locale = lang === "fr" ? "fr-FR" : "en-GB";

const plurals = new Intl.PluralRules(locale);

type Params = Record<string, string | number>;

function fill(text: string, params?: Params): string {
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

export function formatInt(n: number): string {
  return new Intl.NumberFormat(locale).format(n);
}
