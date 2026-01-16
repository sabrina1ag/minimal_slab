# Architecture du Slab Allocator

## Vue d'ensemble

Un slab allocator est un type d'allocateur de mémoire spécialisé conçu pour allouer efficacement de nombreux objets de taille fixe. Il est particulièrement efficace pour les cas d'usage où on alloue et désalloue fréquemment des objets de la même taille.

## Concepts fondamentaux

### 1. Page Allocator

Le **PageAllocator** est la couche de base qui alloue des blocs de mémoire de taille fixe appelés "pages" (généralement 4KB). Dans cette implémentation, le page allocator est simulé et utilise `alloc::alloc` en arrière-plan. Dans un vrai système d'exploitation, ceci serait remplacé par un vrai page allocator qui gère la mémoire physique.

**Responsabilités:**
- Allouer des pages contiguës de mémoire
- Libérer des pages précédemment allouées
- Gérer l'alignement sur les frontières de pages

### 2. Slab

Un **Slab** est un bloc de mémoire (généralement une page) divisé en objets de taille fixe. Chaque slab maintient une liste libre chaînée pour suivre les objets disponibles.

**Structure interne:**
- **memory**: Pointeur vers le début de la mémoire du slab
- **object_size**: Taille de chaque objet
- **num_objects**: Nombre total d'objets dans le slab
- **free_list**: Pointeur vers le premier objet libre (None si plein)
- **allocated_count**: Nombre d'objets actuellement alloués

**Liste libre chaînée:**
Chaque objet libre contient un pointeur vers le prochain objet libre. Quand un objet est alloué, il est retiré de la liste libre. Quand il est libéré, il est remis en tête de la liste libre.

### 3. Slab Cache

Un **SlabCache** est responsable de la gestion de plusieurs slabs pour une taille d’objet donnée.

**Responsabilités :**
- Maintenir une collection de slabs
- Trouver un slab avec de l’espace libre lors d’une allocation
- Créer un nouveau slab lorsque tous les slabs existants sont pleins
- Déléguer les opérations d’allocation et de désallocation aux slabs appropriés

Chaque cache correspond à une taille d’objet spécifique.
