# Activité des Députés — Assemblée nationale 17e législature

Site web "newsroom-grade" affichant des statistiques d'activité parlementaire observable des députés français. Données open data officielles, aucune opinion éditorialie.

**Site en production** : `https://<votre-org>.github.io/activite-deputes/`

---

## Architecture

```
activite-deputes/
├── pipeline/          # ETL Rust — télécharge, parse, agrège, exporte
│   └── src/
│       ├── main.rs        # Orchestrateur
│       ├── downloader.rs  # HTTP + ETags + ZIP
│       ├── parser.rs      # Parsing JSON AN
│       ├── models.rs      # Types normalisés
│       ├── aggregator.rs  # Calcul P30/P180/LEG
│       └── exporter.rs    # JSON + CSV
├── frontend/          # App Leptos WASM (CSR)
│   └── src/
│       ├── lib.rs         # Point d'entrée WASM
│       ├── models.rs      # Types partagés
│       ├── api.rs         # Fetch JSON
│       ├── utils.rs       # Helpers
│       ├── components/    # Layout, KPI, Table, Tooltip…
│       └── pages/         # Home, Députe, Comparer, Export, Méthode
├── site/              # Sortie statique (gitignore sauf data sample)
│   ├── data/          # JSON générés par pipeline
│   └── exports/       # CSV générés par pipeline
└── .github/workflows/ # CI/CD GitHub Actions (cron dimanche 03:00 UTC)
```

## Développement local

### Prérequis

```bash
# Rust stable
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Target WASM
rustup target add wasm32-unknown-unknown

# Trunk (build frontend)
cargo install trunk

# wasm-bindgen
cargo install wasm-bindgen-cli
```

### Étape 1 — Générer les données (pipeline)

```bash
cargo run --release -p pipeline
```

Les fichiers sont générés dans `site/data/` et `site/exports/`.

> ℹ️ Le pipeline télécharge ~200-400 Mo depuis data.assemblee-nationale.fr (1 seule fois, puis ETags pour les mises à jour).

### Étape 2 — Build frontend

```bash
cd frontend
trunk build
# Sortie dans site/
```

Pour le développement avec rechargement automatique :

```bash
cd frontend
trunk serve --port 8080
# ⚠️ Les données doivent être dans ../site/data/ (step 1)
```

### Étape 3 — Serveur local

```bash
cd site
python3 -m http.server 8080
# Ouvrir http://localhost:8080
```

---

## Sources de données

| Dataset | URL |
|---------|-----|
| Députés actifs + organes | [AMO10_...json.zip](http://data.assemblee-nationale.fr/static/openData/repository/17/amo/deputes_actifs_mandats_actifs_organes/AMO10_deputes_actifs_mandats_actifs_organes.json.zip) |
| Scrutins | [Scrutins.json.zip](http://data.assemblee-nationale.fr/static/openData/repository/17/loi/scrutins/Scrutins.json.zip) |
| Amendements | [Amendements.json.zip](http://data.assemblee-nationale.fr/static/openData/repository/17/loi/amendements_div_legis/Amendements.json.zip) |
| Dossiers législatifs | [Dossiers_Legislatifs.json.zip](http://data.assemblee-nationale.fr/static/openData/repository/17/loi/dossiers_legislatifs/Dossiers_Legislatifs.json.zip) |

Licence : **Licence Ouverte v2.0 (Etalab)** — Open Data Assemblée nationale.

---

## Métriques calculées

| Métrique | Définition |
|----------|-----------|
| `participation_rate` | votes_exprimes / scrutins_eligibles (positions enregistrées, **≠ présence physique**) |
| `amd_authored` | Amendements avec le député comme auteur principal |
| `amd_adoption_rate` | Amendements adoptés / amendements déposés |
| `top_dossiers` | Top 10 dossiers par score = 1×votes + 2×amendements |

Fenêtres : **P30** (30j glissants), **P180** (180j glissants), **LEG** (depuis début législature ou mandat).

---

## Déploiement GitHub Pages

1. Forker ce dépôt
2. Aller dans **Settings → Pages → Source : GitHub Actions**
3. Lancer manuellement le workflow (`workflow_dispatch`) pour le premier déploiement
4. Le workflow s'exécute ensuite chaque **dimanche à 03:00 UTC**

### Note sur le `public_url` (important)

Le frontend détecte automatiquement le sous-chemin GitHub Pages via `window.location.pathname`. Si votre repo s'appelle `activite-deputes`, le site sera servi depuis `https://org.github.io/activite-deputes/` et les fetches JSON seront automatiquement préfixés.

Si vous utilisez un domaine personnalisé (CNAME à la racine), aucune configuration supplémentaire n'est nécessaire.

---

## Roadmap

- [ ] Intégration des débats/interventions (syseron.xml)
- [ ] Séries temporelles hebdomadaires (sparklines)
- [ ] Export profil CSV individuel par député
- [ ] Page par groupe parlementaire
- [ ] Comparaison inter-législatures

---

## Licence

Code : MIT. Données : Licence Ouverte v2.0 Etalab (Assemblée nationale).
