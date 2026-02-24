use leptos::*;
use crate::api::{fetch_status, inferred_github_repo_urls};

#[component]
pub fn MethodePage() -> impl IntoView {
    let status = create_resource(|| (), |_| fetch_status());
    let repo_links = inferred_github_repo_urls();
    let repo_url = repo_links.as_ref().map(|(r, _)| r.clone());
    let issue_url = repo_links.as_ref().map(|(_, i)| i.clone());

    view! {
        <div class="reveal" style="max-width:860px;">
            <h1 style="font-size:1.4rem;font-weight:700;margin:0 0 0.4rem 0;">"Méthode & Sources"</h1>
            <p style="color:var(--text-muted);font-size:0.85rem;margin-bottom:2rem;">
                "Ce document décrit précisément ce que mesurent les indicateurs présentés sur ce site, leurs limites, et les sources utilisées."
            </p>

            <Section title="Statut bêta et usage recommandé">
                <p>"Le site est actuellement publié en "<strong>"bêta publique"</strong>" : les principales pages sont opérationnelles, mais certains enrichissements (notamment certains liens AN de PPL et des raffinements UX) sont encore en cours."</p>
                <ul style="padding-left:1.5rem;line-height:1.9;">
                    <li>"Utiliser les vues comme outil d’exploration et de vérification, pas comme verdict automatique."</li>
                    <li>"Toujours croiser avec les sources officielles AN en cas de doute."</li>
                    <li>"Signaler les anomalies de parsing / mapping pour améliorer la couverture."</li>
                </ul>
                <Note>"Les données affichées sont traçables et sourcées, mais certaines associations peuvent rester incomplètes si la source open data évolue ou expose des cas particuliers."</Note>
            </Section>

            <Section title="Module Positions groupes (PPL) — périmètre V1">
                <p>"La page Positions groupes synthétise les propositions de loi (PPL) associées à chaque groupe via les signataires détectés dans les dossiers législatifs."</p>
                <ul style="padding-left:1.5rem;line-height:1.9;">
                    <li>"V1 : "<strong>"PPL d’origine Assemblée uniquement"</strong>" (filtre AN-only) pour éviter les ambiguïtés avec les PPL d’origine Sénat dans la navette."</li>
                    <li>"Une relation groupe↔PPL est marquée comme "<strong>"auteur"</strong>" si au moins un signataire du groupe est auteur principal ; sinon elle est classée en "<strong>"cosignature uniquement"</strong>"."</li>
                    <li>"Le bucket INCONNU regroupe les signataires non mappés au référentiel local (cas résiduels / couverture incomplète)."</li>
                </ul>
                <p>"Cette vue donne un signal de "<em>"ce que les groupes portent / soutiennent"</em>" de façon traçable, mais ne remplace pas une analyse juridique du contenu des textes."</p>
            </Section>

            <Section title="Périmètre et objectif">
                <p>"Ce site agrège et présente des statistiques d'activité parlementaire "
                <em>"observable"</em>" via les données open data publiées par l'Assemblée nationale française. "</p>
                <p>"Il ne contient aucune opinion éditoriale, aucun classement de valeur ni commentaire politique. "</p>
                <p>"Public cible : journalistes, analystes, chercheurs, grand public."</p>
            </Section>

            <Section title="Sources de données">
                <p>"Toutes les données proviennent de "<a href="https://data.assemblee-nationale.fr" target="_blank" rel="noopener" style="color:var(--accent);">"data.assemblee-nationale.fr"</a>", Open Data officiel de l'Assemblée nationale, sous Licence Ouverte v2.0 (Etalab)."</p>
                <p>"Nous utilisons les datasets de la 17e législature :"</p>
                <table class="data-table" style="margin-top:0.75rem;">
                    <thead><tr><th>"Dataset"</th><th>"Contenu"</th><th>"URL"</th></tr></thead>
                    <tbody>
                        <tr>
                            <td style="font-family:monospace;font-size:0.75rem;">"AMO10_deputes_actifs..."</td>
                            <td>"Députés en exercice, mandats, organes (groupes, partis)"</td>
                            <td><a href="http://data.assemblee-nationale.fr/static/openData/repository/17/amo/deputes_actifs_mandats_actifs_organes/AMO10_deputes_actifs_mandats_actifs_organes.json.zip" target="_blank" rel="noopener" style="color:var(--accent);font-size:0.75rem;">"ZIP"</a></td>
                        </tr>
                        <tr>
                            <td style="font-family:monospace;font-size:0.75rem;">"Scrutins.json.zip"</td>
                            <td>"Résultats de l'ensemble des scrutins publics, avec position par acteur"</td>
                            <td><a href="http://data.assemblee-nationale.fr/static/openData/repository/17/loi/scrutins/Scrutins.json.zip" target="_blank" rel="noopener" style="color:var(--accent);font-size:0.75rem;">"ZIP"</a></td>
                        </tr>
                        <tr>
                            <td style="font-family:monospace;font-size:0.75rem;">"Amendements.json.zip"</td>
                            <td>"Tous les amendements déposés, avec auteur(s), sort et dossier"</td>
                            <td><a href="http://data.assemblee-nationale.fr/static/openData/repository/17/loi/amendements_div_legis/Amendements.json.zip" target="_blank" rel="noopener" style="color:var(--accent);font-size:0.75rem;">"ZIP"</a></td>
                        </tr>
                        <tr>
                            <td style="font-family:monospace;font-size:0.75rem;">"Dossiers_Legislatifs.json.zip"</td>
                            <td>"Dossiers législatifs (titres, statuts)"</td>
                            <td><a href="http://data.assemblee-nationale.fr/static/openData/repository/17/loi/dossiers_legislatifs/Dossiers_Legislatifs.json.zip" target="_blank" rel="noopener" style="color:var(--accent);font-size:0.75rem;">"ZIP"</a></td>
                        </tr>
                    </tbody>
                </table>
                <Note>"Note : Le dataset des débats (interventions en séance, syseron.xml.zip) est en cours d'intégration. Les colonnes interventions_count et interventions_chars affichent 0 en V1."</Note>
            </Section>

            <Section title="Fenêtres temporelles">
                <p>"Trois fenêtres sont disponibles :"</p>
                <ul style="padding-left:1.5rem;line-height:2;">
                    <li><strong>"P30"</strong>" : 30 derniers jours glissants (depuis la date de mise à jour)"</li>
                    <li><strong>"P180"</strong>" : 180 derniers jours glissants"</li>
                    <li><strong>"LEG"</strong>" : Depuis le début de la 17e législature (19 juin 2022) ou depuis le début du mandat si le député est entré après"</li>
                </ul>
                <p>"Pour chaque député, la fenêtre effective est l'intersection de la période choisie et de la durée de son mandat. Un député entré en cours de législature n'est comptabilisé que sur les scrutins et amendements postérieurs à son entrée en fonction."</p>
            </Section>

            <Section title="A — Participation aux scrutins publics">
                <p>"Définitions :"</p>
                <ul style="padding-left:1.5rem;line-height:2;">
                    <li><strong>"scrutins_eligibles"</strong>" : Nombre de scrutins publics sur la période où le député avait un mandat actif."</li>
                    <li><strong>"votes_exprimes"</strong>" : Scrutins avec position Pour, Contre ou Abstention."</li>
                    <li><strong>"non_votant"</strong>" : Position NON_VOTANT enregistrée dans le dataset (délégation de vote, absence déclarée…)."</li>
                    <li><strong>"absent"</strong>" : Scrutin éligible sans aucune position enregistrée."</li>
                    <li><strong>"participation_rate"</strong>" = votes_exprimes / scrutins_eligibles."</li>
                </ul>
                <Note>"⚠ Ce n'est pas une mesure de présence physique en hémicycle. Un député peut voter depuis l'une des travées, depuis une salle de vote déportée, ou via délégation selon les règles en vigueur. La position enregistrée dans les données open data est la seule information disponible."</Note>
            </Section>

            <Section title="B — Amendements">
                <ul style="padding-left:1.5rem;line-height:2;">
                    <li><strong>"amd_authored"</strong>" : Amendements où le député figure comme auteur principal (signataire 1)."</li>
                    <li><strong>"amd_cosigned"</strong>" : Amendements où le député est co-signataire (auteur secondaire). Comptabilisé séparément."</li>
                    <li><strong>"amd_adopted"</strong>" : Parmi les amendements de type authored, ceux dont le sort contient 'adopt' (insensible à la casse)."</li>
                    <li><strong>"amd_adoption_rate"</strong>" = amd_adopted / amd_authored (null si amd_authored = 0)."</li>
                </ul>
            </Section>

            <Section title="C — Dossiers législatifs et score d'activité">
                <p>"Pour chaque député, on calcule un score par dossier :"</p>
                <div style="padding:0.75rem 1rem;background:var(--bg-secondary);border:1px solid var(--bg-border);border-radius:6px;font-family:monospace;font-size:0.82rem;margin:0.75rem 0;">
                    "score = 1 × votes_in_dossier + 2 × amd_authored_in_dossier + 1 × interventions_in_dossier"
                </div>
                <p>"Le coefficient 2 sur les amendements reflète un engagement plus actif de rédaction (vs simplement participer au vote). Ce coefficient est arbitraire et documenté ici pour transparence."</p>
                <p>"Seuls les 10 dossiers au score le plus élevé sont affichés par député."</p>
            </Section>

            <Section title="D — Groupes et partis">
                <ul style="padding-left:1.5rem;line-height:2;">
                    <li><strong>"Groupe parlementaire"</strong>" : Regroupement officiel au sein de l'Assemblée nationale. Correspond à l'organe de type GP dans les données."</li>
                    <li><strong>"Parti de rattachement"</strong>" : Organisation politique déclarée par le député (organe de type PARPOL). Peut être absent, non fiable ou différent du groupe."</li>
                </ul>
                <Note>"La distinction entre groupe et parti est importante : un même parti peut avoir des membres dans plusieurs groupes, et un groupe peut rassembler des membres de plusieurs partis."</Note>
            </Section>

            <Section title="Limites de ces mesures">
                <p>"Ces indicateurs ne mesurent pas :"</p>
                <ul style="padding-left:1.5rem;line-height:1.9;">
                    <li>"Le travail local dans la circonscription (permanences, réunions publiques, associations)"</li>
                    <li>"Les réunions de commission (hors scrutins publics)"</li>
                    <li>"Les négociations informelles et le travail de couloir"</li>
                    <li>"La qualité ou l'impact des amendements ou des interventions"</li>
                    <li>"Les rapports rédigés, les missions d'information"</li>
                    <li>"Le contexte politique (votes de groupe, discipline de vote)"</li>
                    <li>"Les absences médicales ou situations personnelles"</li>
                </ul>
                <p>"Un score élevé ou faible ne préjuge pas de la qualité du mandat. Ces chiffres sont des indicateurs d'activité observable, pas des jugements."</p>
            </Section>

            <Section title="Mise à jour automatique">
                <p>"Le pipeline de données est exécuté automatiquement tous les dimanches à 03h00 UTC via GitHub Actions."</p>
                <p>"En cas d'échec du téléchargement ou du parsing d'un dataset, la dernière version publiée est conservée (pas de publication partielle). Les ETags HTTP sont utilisés pour éviter les téléchargements inutiles si les fichiers sources n'ont pas changé."</p>
                {move || status.get().and_then(|r| r.ok()).map(|s| view! {
                    <div style="margin-top:0.75rem;padding:0.6rem 1rem;background:var(--bg-secondary);border:1px solid var(--bg-border);border-radius:6px;font-size:0.78rem;">
                        "Dernière mise à jour réussie : "
                        <strong style="color:var(--text-primary);">{s.last_update_readable}</strong>
                    </div>
                })}
            </Section>

            <section id="retours" style="margin-bottom:2rem;scroll-margin-top:96px;">
                <h2 style="font-size:1rem;font-weight:600;margin:0 0 0.75rem 0;padding-bottom:0.5rem;border-bottom:1px solid var(--bg-border);color:var(--text-primary);">
                    "Retours, bugs et suggestions"
                </h2>
                <div style="line-height:1.7;font-size:0.85rem;color:var(--text-secondary);">
                    <p>"Les retours utilisateurs sont bienvenus pendant la bêta : bugs de parsing, erreurs de mapping, problèmes d’UX, idées de filtres ou vues."</p>
                    <div style="display:flex;gap:.6rem;flex-wrap:wrap;align-items:center;">
                        {match issue_url.clone() {
                            Some(url) => view! {
                                <a href=url target="_blank" rel="noopener noreferrer" class="btn" style="text-decoration:none;">"Créer une issue GitHub ↗"</a>
                            }.into_view(),
                            None => view! {
                                <span style="font-size:.78rem;color:var(--text-muted);">"Lien GitHub non détecté automatiquement (hébergement hors GitHub Pages ou repo privé). Ajoute le lien du repo / issues dans le footer avant publication finale."</span>
                            }.into_view(),
                        }}
                        {match repo_url.clone() {
                            Some(url) => view! {
                                <a href=url target="_blank" rel="noopener noreferrer" style="color:var(--accent);text-decoration:none;font-size:.8rem;">"Voir le code source ↗"</a>
                            }.into_view(),
                            None => view! { <></> }.into_view(),
                        }}
                    </div>
                    <Note>"Pour un signalement utile : indique la page, la période, l’ID député/PPL si possible, et une capture d’écran."</Note>
                </div>
            </section>

            <Section title="Licence et réutilisation">
                <p>"Les données source sont sous Licence Ouverte v2.0 (Etalab). Ce site est un outil de visualisation indépendant, non affilié à l'Assemblée nationale."</p>
                <p>"Le code source est disponible sur GitHub. Les exports CSV/JSON sont librement réutilisables avec attribution de la source (data.assemblee-nationale.fr)."</p>
            </Section>
        </div>
    }
}

#[component]
fn Section(title: &'static str, children: Children) -> impl IntoView {
    view! {
        <section style="margin-bottom:2rem;">
            <h2 style="font-size:1rem;font-weight:600;margin:0 0 0.75rem 0;padding-bottom:0.5rem;border-bottom:1px solid var(--bg-border);color:var(--text-primary);">
                {title}
            </h2>
            <div style="line-height:1.7;font-size:0.85rem;color:var(--text-secondary);">
                {children()}
            </div>
        </section>
    }
}

#[component]
fn Note(children: Children) -> impl IntoView {
    view! {
        <div style="margin-top:0.75rem;padding:0.6rem 0.85rem;background:var(--accent-dim);border-left:3px solid var(--accent);border-radius:0 4px 4px 0;font-size:0.78rem;color:var(--text-secondary);line-height:1.6;">
            {children()}
        </div>
    }
}
