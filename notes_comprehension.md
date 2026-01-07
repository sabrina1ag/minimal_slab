
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


## Ajouter a la definition de cache : 

The slab allocator consists of a variable number of caches that are linked together on a doubly linked circular list called a cache chain. 

A cache, in the context of the slab allocator, is a manager for a number of objects of a particular type like the mm_struct or fs_cache cache and is managed by a struct kmem_cache_s discussed in detail later. The caches are linked via the next field in the cache struct.

On peut avoir une idée des caches disponible en faisant un cat /proc/slabinfo
la structure kmem_cache a les attributs suivant 
cache-name A human readable name such as “tcp_bind_bucket”;
num-active-objs Number of objects that are in use;
total-objs How many objects are available in total including unused;
obj-size The size of each object, typically quite small;
num-active-slabs Number of slabs containing objects that are active;
total-slabs How many slabs in total exist;
num-pages-per-slab The pages required to create one slab, typically 1
la description d’un cache se trouve dans le fichier mm/slab.c 

190 struct kmem_cache_s {
193     struct list_head        slabs_full;
194     struct list_head        slabs_partial;
195     struct list_head        slabs_free;
196     unsigned int            objsize;
197     unsigned int            flags;
198     unsigned int            num;
199     spinlock_t              spinlock;
200 #ifdef CONFIG_SMP
201     unsigned int            batchcount;
202 #endif
203

**description de ces attributs :** 
labs_* These are the three lists where the slabs are stored as described in the previous section;
objsize This is the size of each object packed into the slab;
flags These flags determine how parts of the allocator will behave when dealing with the cache. See Section 8.1.2;
num This is the number of objects contained in each slab;
spinlock A spinlock protecting the structure from concurrent accessses;
batchcount This is the number of objects that will be allocated in batch for the per-cpu caches as described in the previous section.


Cache Static Flags : 
le cache a des flags qui sont set a sa création et ne changeront pas toute sa lifetime. 
2 types de flag, les premiers sont set par le slab allocator : 

CFGS_OFF_SLAB
Indicates that the slab managers for this cache are kept off-slab. This is discussed further in Section 8.2.1
CFLGS_OPTIMIZE
This flag is only ever set and never used


la deuxième catégorie sont set par le créateur du cache 

Flag
Description
SLAB_HWCACHE_ALIGN
Align the objects to the L1 CPU cache
SLAB_MUST_HWCACHE_ALIGN
Force alignment to the L1 CPU cache even if it is very wasteful or slab debugging is enabled
SLAB_NO_REAP
Never reap slabs in this cache
SLAB_CACHE_DMA
Allocate slabs with memory from ZONE_DMA


explication des champs

ya aussi deux autres types de flag 
Cache Dynamic Flags
Cache Allocation Flags

Cache Coloring : 
le cache coloring est une optimisation CPU pas mémoire, Au lieu de faire commencer tous les slabs au même offset mémoire, le slab allocator décale le début des objets dans chaque slab, le calcul est effectué a la creation du cache kmem_cache_create
Voyons un exemple de la détermination du coloring : 
Exemple expliqué pas à pas

Hypothèses :
Adresse de base du slab (s_mem) = 0
100 bytes inutilisés dans le slab
Alignement L1 = 32 bytes


Déductions :
Offsets possibles : 0, 32, 64, 96
colour = 3
colour_off = 32


Allocation des slabs :
Slab
Offset de départ
Slab 1
0
Slab 2
32
Slab 3
64
Slab 4
96
Slab 5
0 (wrap-around)


Cette notion est compliquée, et c’est une des raisons de l’apparition du SLUB. 

Cache Creation : 
The function kmem_cache_create() is responsible for creating new caches and adding them to the cache chain. Parmis les tâches effectuées, on peut citer : 
cache coloring qu’on vient d'expliquer-
Initialise remaining fields in cache descriptor;
Add the new cache to the cache chain.
Align the object size to the word size;



Cache Reaping et Cache Shrinking (SLAB)
les caches slabs ne se réduisent pas automatiquement quand le slab devient inutilisé, Lorsqu’un slab est libéré, il est simplement placé dans la liste slabs_free pour une réutilisation future. Lorsque le système manque de mémoire, le démon kswapd déclenche un mécanisme appelé cache reaping, dont le rôle est de libérer des pages mémoire en forçant certains caches à relâcher leurs slabs inutilisés.
Cache Reaping
Le reaping consiste à sélectionner un cache candidat et à lui demander de réduire sa consommation mémoire.
 La sélection ne tient pas compte des nœuds NUMA ni des zones mémoire (c’est quoi NUMA), ce qui peut entraîner une libération de mémoire dans des régions non sous pression. Ce comportement est acceptable sur des architectures simples comme x86.
Lors du reaping :
seuls quelques caches sont examinés à chaque itération ;


les caches récemment agrandis ou en cours de croissance sont évités ;


les caches capables de libérer le plus de pages sont privilégiés ;


une partie des slabs présents dans slabs_free est libérée.



Cache Shrinking
Une fois un cache sélectionné, le shrinking est volontairement simple et agressif :
les caches per-CPU sont vidés ;


Les slabs présents dans slabs_free sont libérés.

**Deux variantes existent :**
une fonction destinée aux utilisateurs du slab allocator, qui libère les slabs libres ;

une fonction interne, utilisée lors de la destruction complète d’un cache, qui garantit que le cache est entièrement vidé.

Destruction d’un cache
Lorsqu’un module est déchargé, les caches qu’il a créés doivent être détruits explicitement.
 La destruction d’un cache implique :
le retrait du cache de la liste globale ;


la libération de tous les slabs associés ;

la suppression des structures per-CPU (expliqué a la section 3 ) ;
la libération du descripteur du cache.

Le code kernel central ne détruit généralement pas ses caches, car ils existent pendant toute la durée de vie du système.
Slabs : 
Un slab est décrit par la structure slab_t, volontairement simple, mais qui pilote une organisation interne plus complexe.
typedef struct slab_s {
    struct list_head  list;
    unsigned long     colouroff;
    void              *s_mem;
    unsigned int      inuse;
    kmem_bufctl_t     free;
} slab_t;


**Association objet → slab → cache**
Il n’existe pas de pointeur direct depuis un objet vers son slab ou son cache.
 Cette relation est reconstruite via la struct page associée aux pages physiques du slab :
Les champs next et prev de page->list sont utilisés pour stocker des références vers :


le cache (SET_PAGE_CACHE)
le slab (SET_PAGE_SLAB)
Les macros GET_PAGE_CACHE() et GET_PAGE_SLAB() permettent de retrouver ces descripteurs.


Ce mécanisme relie donc page → slab → cache sans métadonnées supplémentaires dans les objets.
création d’un slab : 
de nouveaux slabs sont allocés à un cache via la fonction kmem_cache_grow(), on appelle ça generalement du “cache gromwing” et ca arrive quand il ny a pas d’objet libre dans nr_partial et pas de slabs libre dans slabs_free.
les etapes de creation d’un slab comprennent; le calcul du color offset, allocation de la memoire et du descripteur du slab, association des pages aux slabs et au cache et l’ajout des slabs dans le cache. 

le slab allocator doit pouvoir trouver rapidement les objets libre dans les slabs partiellement remplis, pour cela il utilise la structure kmem_bufctl_t (un tablea d’unsigned integer), le nombre d’element dans le tableau est le meme que le nombre d’objet dans la slab. 
l’access au tableau se fait via la macro #define slab_bufctl(slabp) ((kmem_bufctl_t *)(((slab_t*)slabp)+1))
car il ny pas de pointeur sur le premier element et ce tableau est stocké apres le slab descripteur. 

exemple d’un tableau avec 5 objet, le dernier element est toujours BUFCTL_END, ce tableau est intialisé au debut de l’initilisation du cache. 
Quand on alloue un objet c’est la fonction kmem_cache_alloc()qui se charge de mettre a jour ce tableau, elle se base sur  slab_t→free qui contient l’index du premier objet libre, le tableau n’est pas modifie les entrées deviennent juste inaccessible. 





exemple : un objet de 0x100 octet, contient le pointeur offset 0x80 pour le prochain objet libre. 

# Source : 
https://www.kernel.org/doc/gorman/html/understand/understand011.html#Sec:%20Per-CPU%20Object%20Cache


