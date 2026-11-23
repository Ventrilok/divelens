# Textes de l'interface : un fichier par langue — plan d'implémentation

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** sortir les textes anglais et français de `app/src/i18n.ts` dans `app/src/i18n/en.ts` et `app/src/i18n/fr.ts`, sans changer un seul texte ni le comportement.

**Architecture:** `en.ts` porte l'objet anglais (`as const`) et en dérive le type `Key` : l'anglais reste la référence. `fr.ts` est un littéral objet typé `Record<Key, string>` : le compilateur refuse une clé manquante ou en trop, comme aujourd'hui. `i18n.ts` ne garde que la mécanique (choix de la langue, pluriels, paramètres) et ré-exporte `Key`, donc aucun écran ne change ses imports. Le test Rust qui compare les noms d'annulation du plug-in aux textes lit désormais chaque langue dans son fichier.

**Tech Stack:** TypeScript 6 (`tsc` dans `npm run build`), Vite/Vitest, test d'intégration Rust (`cargo test -p divelens`).

**Spec:** issue GitHub #58, « refactor(app): split interface texts into one file per language » (`gh issue view 58`).

## Global Constraints

- Aucun texte ne change, aucune langue n'est ajoutée : rien de visible pour un plongeur.
- Hors périmètre : les descriptions d'applications dans `core/`, `TranslatedStrings_fr.txt` du plug-in, les installeurs.
- Fichiers TypeScript, pas JSON : le garde-fou reste une vérification de build.
- Les écrans continuent d'importer `t`, `tn`, `Key`, etc. depuis `i18n.ts` : aucun import d'écran ne change.
- Commit de référence pour le déplacement : `47d4b8f` (main). Entrées anglaises = lignes 7–370 de `app/src/i18n.ts`, entrées françaises = lignes 376–739 (364 lignes chacune, lignes vides de séparation et entrées sur deux lignes comprises).
- Branche `refactor/58-split-interface-texts` ; commits Conventional Commits ; push et PR seulement à la demande de pb.

## Review Focus

1. **Un texte modifié pendant le déplacement** (accent, apostrophe, retour à la ligne) : les deux blocs sont copiés par extraction de lignes depuis git, jamais retapés, et un `diff` contre `47d4b8f` doit être vide (Task 1, étape 5).
2. **Garde-fou affaibli** (français typé avec spread, cast, `Partial`, `satisfies` partiel, ou `Key` dérivé du français) : vérification à la main dans les deux sens, clé retirée puis clé inconnue, `npm run build` doit échouer (Task 1, étape 6).
3. **Import ambigu fichier/dossier** : `../i18n` doit continuer de désigner `i18n.ts`. Pas de `i18n/index.ts`. `npm run build` et `npm test` le prouvent (Task 1, étape 7).
4. **Cycle d'import à l'exécution** : les fichiers de langue n'importent que des types, et seulement depuis `./en`, jamais depuis `../i18n` (Task 1, étape 5).
5. **Test Rust qui compare la mauvaise langue** : le test lit l'anglais dans `en.ts` et le français dans `fr.ts`, chacun dans son fichier, plus de « première occurrence / dernière occurrence » (Task 1, étapes 1–2).

---

### Task 1: Un fichier de textes par langue

**Files:**
- Create: `app/src/i18n/en.ts`
- Create: `app/src/i18n/fr.ts`
- Modify: `app/src/i18n.ts` (lignes 1–742 remplacées par un en-tête ; lignes 743–807 inchangées)
- Modify: `app/src-tauri/tests/apply_contract.rs` (test `the_undo_names_match`, lignes 104–121)

**Interfaces:**
- Consumes: rien.
- Produces:
  - `app/src/i18n/en.ts` : `export const en = { … } as const;` et `export type Key = keyof typeof en;`
  - `app/src/i18n/fr.ts` : `export const fr: Record<Key, string> = { … };` avec `import type { Key } from "./en";`
  - `app/src/i18n.ts` : exports inchangés — `Lang`, `Key` (via `export type { Key }`), `systemLang`, `lang`, `locale`, `setLanguage`, `localized`, `fill`, `t`, `tn`, `formatInt`.

- [ ] **Step 0: Préparer la branche**

Créer la branche `refactor/58-split-interface-texts` depuis `main` (worktree via superpowers:using-git-worktrees si utilisé). Y committer ce plan seul :

```bash
git add docs/superpowers/plans/2026-11-23-split-interface-texts.md
git commit -m "docs: plan the split of interface texts per language"
```

Si `app/dist` n'existe pas : `cd app && npm ci && npm run build`.

- [ ] **Step 1: Faire lire au test Rust un fichier par langue (test d'abord)**

Dans `the_undo_names_match`, remplacer la lecture de `app/src/i18n.ts` et la closure `line_of(key, french)` par :
- `let english = repo_file("app/src/i18n/en.ts");` et `let french = repo_file("app/src/i18n/fr.ts");`
- une closure `line_of(texts: &str, file: &str, key: &str) -> String` qui renvoie la ligne (trimée) commençant par `"<key>":`, et panique avec `no text {key} in {file}` sinon ;
- les deux assertions appellent `line_of(&english, "app/src/i18n/en.ts", i18n_key)` et `line_of(&french, "app/src/i18n/fr.ts", i18n_key)` ; leurs messages restent ceux d'aujourd'hui.
- Le commentaire « The i18n has an English block, then a French one… » devient : chaque langue est dans son propre fichier.

Les variables `english`/`french` existent déjà dans la boucle (noms d'annulation) : nommer les textes `english_texts`/`french_texts` pour éviter l'ombrage.

- [ ] **Step 2: Vérifier que le test échoue**

Run: `cargo test -p divelens --test apply_contract the_undo_names_match`
Expected: FAIL, panique `cannot read ".../app/src/i18n/en.ts"`.

- [ ] **Step 3: Créer `en.ts` et `fr.ts` par extraction depuis git**

Les corps viennent de `47d4b8f`, sans retouche :

```bash
mkdir -p app/src/i18n
{
  printf '%s\n' \
    '// English interface texts: the reference. Every other language gives' \
    '// exactly these keys, or the build fails.' \
    '' \
    'export const en = {'
  git show 47d4b8f:app/src/i18n.ts | sed -n '7,370p'
  printf '%s\n' '} as const;' '' 'export type Key = keyof typeof en;'
} > app/src/i18n/en.ts
{
  printf '%s\n' \
    '// French interface texts. Same keys as en.ts; keep every {placeholder} as' \
    '// it is. Keys ending in _one and _other are the singular and plural forms.' \
    '' \
    'import type { Key } from "./en";' \
    '' \
    'export const fr: Record<Key, string> = {'
  git show 47d4b8f:app/src/i18n.ts | sed -n '376,739p'
  printf '%s\n' '};'
} > app/src/i18n/fr.ts
```

- [ ] **Step 4: Réduire `i18n.ts` à la mécanique**

Remplacer les lignes 1–742 de `app/src/i18n.ts` (en-tête, les deux blocs de textes, `const dictionaries`) par cet en-tête ; les lignes 743–807 (de `systemLang` à `formatInt`) restent à l'identique :

```ts
// How the interface speaks: the language (the system's unless chosen in
// Settings, changed at once, spec F2), plurals and placeholders. The texts
// are in i18n/, one file per language; English is the reference.

import { en, type Key } from "./i18n/en";
import { fr } from "./i18n/fr";

export type { Key };
export type Lang = "en" | "fr";

const dictionaries: Record<Lang, Record<Key, string>> = { en, fr };
```

Ne pas créer `app/src/i18n/index.ts`.

- [ ] **Step 5: Vérifier que rien n'a bougé**

```bash
diff <(git show 47d4b8f:app/src/i18n.ts | sed -n '7,370p') <(sed -n '5,368p' app/src/i18n/en.ts)
diff <(git show 47d4b8f:app/src/i18n.ts | sed -n '376,739p') <(sed -n '7,370p' app/src/i18n/fr.ts)
diff <(git show 47d4b8f:app/src/i18n.ts | sed -n '743,807p') <(sed -n '12,76p' app/src/i18n.ts)
grep -n "import" app/src/i18n/*.ts
grep -c '": ' app/src/i18n.ts
```

Expected : les trois `diff` sans sortie ; le seul import des fichiers de langue est `import type { Key } from "./en";` dans `fr.ts` ; le dernier `grep` renvoie 0 (plus aucun texte dans `i18n.ts`). Si un numéro de ligne ne tombe pas juste, corriger l'en-tête, pas les textes.

- [ ] **Step 6: Vérifier le garde-fou dans les deux sens**

`git add app/src/i18n` d'abord, pour pouvoir restaurer `fr.ts` avec `git checkout app/src/i18n/fr.ts`. Puis, dans `app/` :
- retirer la ligne `"nav.soon": …` de `fr.ts`, `npm run build` : échec attendu, « Property '"nav.soon"' is missing… » ; restaurer ;
- ajouter `"nav.unknown": "x",` dans `fr.ts`, `npm run build` : échec attendu, « Object literal may only specify known properties… » ; restaurer.

Rejouer ensuite le `diff` de l'étape 5 : toujours vide.

- [ ] **Step 7: Lancer toutes les vérifications de l'app**

```bash
cargo fmt --all --check
cd app && npm test && npm run build && cd ..
cargo clippy -p divelens --all-targets -- -D warnings
cargo test -p divelens
```

Expected : tout passe, y compris `the_undo_names_match`.

- [ ] **Step 8: Commit**

```bash
git add app/src/i18n.ts app/src/i18n/en.ts app/src/i18n/fr.ts app/src-tauri/tests/apply_contract.rs
git commit -m "refactor(app): split interface texts into one file per language"
```

Corps du commit : « Closes #58. » puis la ligne d'attribution.

---

### Après la tâche : pull request (sur demande de pb)

Titre : `refactor(app): split interface texts into one file per language`. Modèle de PR rempli :
- **Summary** : textes anglais et français déplacés dans `app/src/i18n/en.ts` et `fr.ts`, `i18n.ts` ne garde que la mécanique ; le test du contrat Undo lit chaque langue dans son fichier. `Closes #58`.
- **Checks run** : fmt + ligne « App, front end or plug-in » cochées ; « Tried in Lightroom » non coché : aucune modification du plug-in ni des écritures dans le catalogue, donc aucun test de `docs/contributing/lightroom-checks.md` ne s'applique.
- **Visible choices** : None.
