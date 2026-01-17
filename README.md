# minimal_slab

Un allocateur de mémoire de type slab minimal implémenté en Rust pour environnements `no_std`.

## Description

Ce projet pour le module Rust implémente un slab allocator, un type d'allocateur de mémoire efficace pour allouer et désallouer des objets de taille fixe (taille defini par le cache).

### Architecture
Le projet est organisé en plusieurs modules:

- **`page_allocator`**: Un allocateur de pages simulé qui alloue des blocs de mémoire de 4KB (pages)
- **`slab`**: Un slab individuel qui divise une page en objets de taille fixe
- **`slab_cache`**: Un cache qui gère plusieurs slabs (minimum 2) et alloue automatiquement de nouveaux slabs quand nécessaire
- **`slab_allocator`**: L'interface principale qui expose le slab allocator avec plusieurs caches pour différentes tailles

### Fonctionnement

1. **Allocation**: Quand un objet est demandé, le slab allocator sélectionne le cache approprié selon la taille. Si tous les slabs du cache sont pleins, un nouveau slab est automatiquement alloué.

2. **Désallocation**: Quand un objet est libéré, il est remis dans la liste libre du slab correspondant et peut être réutilisé pour de futures allocations.

3. **Liste libre**: Chaque slab maintient une liste libre chaînée où chaque objet libre contient un pointeur vers le prochain objet libre.

## Compilation

**Prérequis**
- Rust (version 1.70 ou supérieure)
- Cargo

**Compiler le projet**
- cargo build

Pour compiler en mode release:
- cargo build --release

**Vérifier le code**

```bash
cargo check
```

**Tests**
Le projet inclut :

des tests unitaires (src/lib.rs)

des tests d’intégration (tests/)

Pour lancer tous les tests :
```bash
cargo test
```
**Formatage (rustfmt)**

Vérifier le formatage :
```bash
cargo fmt -- --check
```

Appliquer le formatage :
```bash
cargo fmt
```

Lint (clippy)

Lancer clippy sur tous les targets (lib + tests) :
```bash
cargo clippy --all-targets
```
**Tests Miri (détection d’UB)**

Miri détecte certains comportements indéfinis (use-after-free, aliasing invalide, etc.).

Installer Miri :
```bash
rustup +nightly component add miri
```

Lancer les tests avec Miri :
```bash
cargo +nightly miri test --lib
```
## Sécurité

### Documentation des blocs unsafe

Tous les blocs `unsafe` dans ce code sont documentés avec des sections `# Safety` expliquant:
- Les préconditions que l'appelant doit garantir
- Les garanties que le code fournit
- Les risques potentiels (double-free, use-after-free, etc.)
```
## Limitations

- Le slab allocator ne supporte actuellement que les allocations jusqu'à 256 octets
- Le cache de slabs est limité à 2 slabs (pour une simulation minimale)
- Le page allocator est simulé et utilise `alloc::alloc` en arrière-plan
