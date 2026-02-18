# Dolt Migration — Notes internes

## Contexte

Depuis bd >= 0.50, le backend Dolt remplace SQLite. Les projets existants doivent migrer.

## Ce qui s'est passe sur beads-task-issue-tracker (2026-02-18)

### Symptome
- Apres mise a jour vers bd 0.52, le projet affichait une page vide (0 issues)
- `bd list` retournait `[]` sans erreur
- Un dossier `.beads/dolt/` existait (migration partielle) mais `.beads/.dolt` manquait

### Cause
bd 0.52 a declenche une migration partielle automatiquement (probablement lors d'une ecriture).
Le dossier `dolt/` a ete cree mais la migration n'a pas abouti. bd lisait depuis Dolt (vide) au lieu de SQLite.

### Procedure de recuperation

```bash
# 1. Verifier que le JSONL contient les issues (source de verite)
wc -l .beads/issues.jsonl

# 2. Tuer tout processus bd/dolt qui verrouillerait la base
pkill -f "bd doctor"; pkill -f "bd daemon"; pkill -f dolt
rm -f .beads/daemon.lock .beads/daemon.pid .beads/bd.sock .beads/dolt-access.lock .beads/.jsonl.lock

# 3. Supprimer le dolt corrompu/partiel et la SQLite corrompue
rm -rf .beads/dolt .beads/beads.db .beads/beads.db-shm .beads/beads.db-wal

# 4. Reinitialiser avec le bon prefixe (verifier dans issues.jsonl)
bd init --prefix beads

# 5. Filtrer les issues tombstone (bd 0.52 les rejette) puis importer
python3 -c "
import json
with open('.beads/issues.jsonl') as f:
    with open('/tmp/beads-clean.jsonl', 'w') as out:
        for line in f:
            line = line.strip()
            if not line: continue
            try:
                issue = json.loads(line)
                if issue.get('status') == 'tombstone': continue
                out.write(json.dumps(issue) + '\n')
            except: continue
"
bd import -i /tmp/beads-clean.jsonl

# 6. Verifier
bd count
bd list --limit=5
```

### Points d'attention

- **Le JSONL est la source de verite** : tant qu'il est intact, les issues sont recuperables
- **Les issues "tombstone"** (106 dans notre cas) sont des issues supprimees. bd 0.52 les rejette a l'import, c'est normal
- **Le prefixe doit correspondre** : `bd init --prefix X` doit matcher le prefixe des issue IDs dans le JSONL (ex: `beads-0bn` → prefixe `beads`)
- **Les verrous Dolt** : `bd doctor` peut creer un verrou et se bloquer lui-meme. Toujours tuer les processus existants avant de retenter
- **`bd migrate --to-dolt`** necessite la base SQLite. Si elle a ete supprimee, utiliser `bd init` + `bd import` a la place

## Risque pour les autres projets

La migration partielle peut arriver si :
1. bd 0.52 est installe
2. Une commande d'ecriture (`bd update`, `bd create`, etc.) est executee sur un projet SQLite
3. La migration automatique echoue ou est interrompue

**Indicateurs** :
- Dossier `.beads/dolt/` present mais `bd list` retourne `[]`
- Fichier `.beads/dolt-access.lock` present
- `bd list` retourne une erreur "Dolt backend configured but database not found"

## Detection dans l'application

L'app detecte 3 cas :
1. **Erreur explicite** : "Dolt backend configured but database not found" → intercepte par `isDoltMigrationError()` dans les catch
2. **Detection proactive** : `bd_check_needs_migration` verifie si bd >= 0.50 + projet pas Dolt + donnees existantes
3. **Migration partielle** : dossier `dolt/` present mais `.dolt` manquant

La commande `bd_migrate_to_dolt` nettoie un dossier `dolt/` partiel avant de relancer `bd migrate --to-dolt --yes`.
