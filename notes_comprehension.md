
Kernel Constraint : 

Pourquoi le kernel linux a besoin d’une tout autre heap ? 
Cela se justifie par le fait que le kernel doit être : 
memory efficient  : aussi, le kernel tourne en boucle, c’est pas un programme userland, du coup restarting the kernel c’est essentiellement reboot la machine, donc le kernel doit être memory efficient, il peut pas perdre l’indice en mémoire ou avoir des fuites de mémoires. 
performant

le glibc est le GNU de linux, ya le userland inclus, et donc ya bcp de meta data (reading, writing), (size, previous size, pointer, nextpointer, large bins)

on perds bcp de temps à bouger ces free chunks entre plusieurs bins, 
pareil si je fais un malloc et que j’ai un chunks plus gros que ce que j’ai besoin le gbc est capable de le diviser en deux et donc peut etre qu’il ya une meilleure implémentation de la heap qui pourra m'éviter ce problème

Limiter la fragmentation : Les allocations/désallocations répétées de tailles variables créent des "trous" dans la mémoire.

Regardons ensemble un exemple de fragmentation : 

char *my_buf = mallonc(0x80);
free(my_buf)
my_buf = malloc(0x60);


ici il ya de grande chances que le chunks qu’on vient de free sera utilisé pour le second malloc et donc on sépare le grand chunks en deux sous chunks un de 60 et un de 20, si on boucle sur cette opération on aura plusieurs chunks de 20, qui ne serviront à rien, c’est de la fragmentation de mémoire, et c’est quelque chose que le kernel, un programme qui tourne indéfiniment voudrait éviter. 

Les allocateurs de slab sont des mécanismes qui aident a gerer les allocations memoires et resoudre les soucis enoncés




Slab Allocators : 

Dans ce document nous allons utiliser les normes de nomination standard : slab en minuscule signifie un espace mémoire contiguë (incluant les 3 types)  et SLAB en majuscule signifie une implémentation spécifique du slab générique. 

Définition du Slab Allocators : 
Un slab allocator pré-alloue des blocs de mémoire (slabs) contenant plusieurs objets de la même taille. Ces objets sont organisés en listes pour une allocation/désallocation rapide.

Vue d’ensemble : SLOB, Slob, Slub

Le kernel linux a évolué à travers 3 generation d'allocations de Slab : 

Historique des 3 types de slab allocators : 
slob (Simple List of Blocks) “As compact As Possible” : 
obsolète mais toujours utilisé dans des endroits spécifique comme le embeded systems
vient du livre K&R et c’était celui utilisé de 1991-1999, c’est compact mais c'était pas efficient, très utile pour les espace memoires petit, et c’est encore utilisé aujourd’hui pour les devices avec des espaces mémoires petit. 
Pas de Structure de cache complexe (notion exploré un peu plus bas dans ce document)
La Fragmentation reste élevé


SLAB Solaris type allocator (1999-2008) : on peut compiler avec mais ce n’est pas le mode par défaut, cet allocator est benchmark friendly, il est souvent désigné comme “as cache friendly as possible”, ça se voit dans le Kmem_cache qui est structuré comme suit : 
**Structure :**
```
kmem_cache
  ├── slabs_full (liste de slabs complètement alloués)
  ├── slabs_partial (liste de slabs partiellement alloués)
  └── slabs_free (liste de slabs vides)
```



slub : le mode par défaut aujourd’hui, (uncured slab allocators) pour les raisons suivantes : 
Simple and instruction cost counts. 
Superior Debugging. 
Defragmentation. 
Execution time friendly


Timeline - Slab Subsystem Développement - Crédit : Christoph Lameter, LinuxCon/Düsseldorf 2014

Slab allocator – terminologie kernel
Dans un système d’exploitation, la heap du kernel est différente de la heap du userland, nous allons d’abord expliquer la terminologie avant d’expliquer la data structures des allocateurs : 

caches : gère les objets de taille fixe, un cache contient plusieurs slabs,Exemples :


cache pour objets de 256 octets
cache pour objets de 512 octets
Il est aussi possible de créer des caches personnalisés pour un type précis 

Slabs : c’est un bloc de mémoire contiguë, une ou plusieurs pages physiques libres, un slab est composé de slots (emplacement de taille fixe taille défini par le cache)), un slab appartient toujours à un seul cache. 

Slots : est une région mémoire de taille fixe dans le slab, sa taille est fixé par le cache; managé par le cache, donc si le cache est 256 o, les slots seront chacun 256o, un slot peut être : 
libre 
occupé par un objet kernel
exemple : dans le cas d’un kmalloc le pointeur retourné est sur l’un de ces slots disponible, quand il sera utilisé on dira que ce slots contient un objet (un kernel objet)

 schema pwn.college, schema conceptuelle
exemple : 
Page = 4096 bytes
Cache = 256 bytes
Slots par slab = 4096 / 256 = 16

Slab Allocators - Schéma Technique

le cache (kmem_cache) utilise deux types de structures 
kmem_cache_cpu 
kmeme_cache_node


Figure - Schema Technique Slab Allocators - Source : pwn.college  Kernel Exploitation - Slab Allocators


Dans cette figure, ce qui est a gauche (cpu) peut être considéré comme le “working” slabs in use.
Ce qui est à droite c’est ce qui n’est pas actuellement utilisé par le actuel cpu

Pourquoi  as-on une division ?
 c’est exactement comme dans le userland avec les threads, on ne veut pas avoir des threads qui s’attendent entre eux donc, et ça nous permet de savoir ce que le CPU actuel utilise sans avoir de la contention avec d’autres cpu. 

pour chaque cache pour chaque cpu il ya une active working slab, quand on a besoin d’un new new slab, le kmem_cache va regarder à droite, et claim ownership sur un slab libre, le slab aura surement qlq slots alloué, mais il pourra être utilisé. 

on gardera la trace des slabs avec des slots libre dans une liste nr_partial dans le kmem_cache_node

Slab Memory - Slots : Comment les objets sont alloués dans le slab, dans les slots et comment ces slots sont reliés ? 

Quand on free un objet, c’est pushed on a single linked list, si on free un second objet, ca fait un push à la tête de la liste et ca change les pointeurs. 
kmem_cache_cpu pointe sur la tete (second object libère) et le second objet pointe sur le suivant (qui lui pointe sur null vu qu’on a seulement deux objets libres). 

Schématisation de l’exemple : 


un des deux objets se libère

on pointe sur la tête de la liste des slots vides (en LIFO), si un second objet se libere, ca donnera une liste chainé et dans le slots libre une partie des bytes contient l’adresse du pointeur suivant. 




exemple : un objet de 0x100 octet, contient le pointeur offset 0x80 pour le prochain objet libre. 




