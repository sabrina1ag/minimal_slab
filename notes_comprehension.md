
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

expliquer c’est quoi un kernel object ;kernel object (fd, semaphores, file objects)
Ce modèle a également des implications importantes en matière de sécurité, notamment dans le contexte des vulnérabilités de type use-after-free, heap overflow ou exploitation du kernel heap.










Pourquoi Le Slub Allocator remplace le SLAB ? 

Les évolutions des versions dans SLAB ont fixé beaucoup de bug notamment ; 
tout ces bugs multiple due à la complexité du code présent dans mm/slab.c, le Slub allocateur vise à adresser ces nombreux concertation dans l'implémentation existante. 

Gestion complexe des files d’objets
SLAB repose sur de nombreuses files d’objets (object queues) réparties par CPU et par nœud NUMA.Cette gestion est complexe et coûteuse. SLUB supprime complètement ces files : chaque CPU travaille directement sur son slab actif, sans mise en file intermédiaire des objets. Cela simplifie fortement le chemin d’allocation et de libération.

Surcharge mémoire des files d’objets
Dans SLAB, les files d’objets existent :par CPU,par nœud NUMA,et même sous forme de files “alien” pour les accès distants. Sur de très grands systèmes (centaines ou milliers de CPU/nœuds), cette multiplication entraîne une explosion du nombre de files et des références stockées. Plusieurs gigaoctets de mémoire peuvent être consommés uniquement pour gérer ces structures, sans compter les objets eux-mêmes.
SLUB élimine ces files, réduisant drastiquement la consommation mémoire et supprimant ce
risque de dérive.

Surcharge de métadonnées dans SLAB : 
SLAB stocke ses métadonnées au début de chaque slab.
 Cela empêche un alignement naturel des objets en mémoire et entraîne un gaspillage d’espace.SLUB déplace toutes les métadonnées dans la structure struct page.
 Les objets peuvent ainsi être naturellement alignés (par exemple un objet de 128 octets aligné sur 128 octets) et remplir une page sans perte, ce que SLAB ne permet pas.

Reaping Complexe : 
le mécanisme de cache reaping est complexe sous SLAB, Slub le simplifie comme suit : 
sur un système mono-processeur, le reaping est inutile ;
Sur un système SMP, un slab per-CPU est simplement replacé dans une liste de slabs partiels, sans parcours d’objets.

Simplification de la politique NUMA : 
SLAB applique les politiques NUMA au niveau des objets individuels, ce qui implique de fréquents accès aux politiques mémoire et peut entraîner des alternances coûteuses entre nœuds.
SLUB délègue entièrement la gestion NUMA à l’allocateur de pages.



Accumulation des slabs partiels
SLAB maintient des listes de slabs partiels par nœud NUMA.
 Avec le temps, ces listes peuvent grossir, augmentant la fragmentation, car ces slabs ne peuvent être réutilisés que par des allocations sur le même nœud.SLUB utilise un pool global de slabs partiels, ce qui permet de réutiliser plus efficacement les slabs et de réduire la fragmentation.
Fusion des caches (slab merging)
SLAB maintient de nombreux caches similaires sans mécanisme de regroupement. SLUB détecte les caches équivalents au démarrage et les fusionne automatiquement.
En pratique, jusqu’à 50 % des caches peuvent être éliminés, ce qui :
améliore l’utilisation mémoire ;
réduit la fragmentation ;
permet de remplir à nouveau des slabs partiellement alloués.
Diagnostics
Les outils de diagnostic SLAB sont difficiles à utiliser et nécessitent une recompilation du kernel. SLUB intègre nativement des mécanismes de debug activables dynamiquement (slab_debug), sans pénaliser les chemins critiques.

Resiliency
Avec les vérifications de cohérence activées, SLUB peut détecter des erreurs courantes (corruption, double free, etc.) et tenter de maintenir le système opérationnel..

Tracing
SLUB permet le traçage précis des opérations sur un cache donné (slab_debug=T), incluant l’état des objets lors de leur libération, ce qui est précieux pour l’analyse post-mortem.

Performance increase
Les benchmarks montrent des gains de performance de l’ordre de 5 à 10 % sur des charges comme kernbench.
Avec des slabs de plus grande taille et une meilleure gestion de la fragmentation, SLUB offre un potentiel de scalabilité supérieur à SLAB.

https://lwn.net/Articles/229096/









actuellement dans le kernel il ny a plus que le slub allocator, mais si on target un ancien kernel c’est utile a savoir. 

un vulnerable object c’est un objet affecté par une sorte de vulnérabilité. 

D’un point de vue exploit on peut faire quoi ? 
comme un out of bounds access, si le slots d'après est vide : 

 on peut accéder au métadonnées de cet objet vide et donc les compromettre, si cette objet n’est pas vide, ça reste très tricky à implémenter, ceci sera expliqué plus bas. 

Seconde attaque possible c’est le fait de corrupt some other object so you can imagine that we have if we have another object that contains for example a function pointer and we're able to override the function pointer why out of bounds res of to three and then make the kernel trigger that function point we might be able to hijack the control flow of the kernel execution and n like escalate privileges as an example and this is a much easier approach

Another object ■ For example, leak vmlinux address to bypass KASLR ■ Or gain a better primitive (Arbitrary Address Execution or AARW) to escalate privileges ■ Spoiler: easy and thus the go-to approach

Pour faire du Slub Memory Corruption nous avons besoin de réunir 3 conditions : 

avoir un kernel bug qui provoque un out-of-bounds acces au objet vulnerable (nous n’allons pas aborder comment trouver un kernel bug dans ce document, nous allons supposer qu’on a deja un kernel bug)

Trouver un objet cible (il existe des writeup sur les objets cible du kernel) sur lequel on pourrait faire un exploit ou un data leaks. 

3eme on doit Need to shape (aka groom or massage) Slab memory ○ For OOB: Put vulnerable and target object next to each other ○ For UAF: Put target object into slot of freed vulnerable object. 
need to figure out how to put the vulnerable and the target object next next to each other and for use of to free we'll need to figure out how to put Target object into a slot that is referenced by use of to free reference
Nous allons nous centrer sur le 3eme point pour expliquer les exploits : 

Pour les exploits on suppose que les parametres sont assez defaut, on se concentre on single cache corruption. 

Expliquer SLUB : jusqua 64 [ jusqua 83 allocation | exploit
cache :

struct kmem_cache      

struct kmem_cache { 
// Per-CPU cache data: 
struct kmem_cache_cpu __percpu *cpu_slab; 
// Per-node cache data: 
struct kmem_cache_node *node[MAX_NUMNODES];
 ... 
const char *name;  // Cache name slab_flags_t 
flags;  // Cache flags unsigned 
int object_size; // Size of objects unsigned
int offset; // Freelist pointer offset                                                
unsigned long min_partial; 
unsigned int cpu_partial_slabs;
 }; 


struct kmem_cache_cpu __percpu *cpu_slab; il ya une instance de ce pointeur pour chaque per cpu dans le systeme, il est lié a des données concernant un cpu, et la raison derriere ça c’est une question de performance, on aimerait pas avoir plusieurs variables communes pour plusieurs CPU. 

struct kmem_cache_node *node[MAX_NUMNODES]; il ya une instance de ce pointeur pour chaque NUMA (non uniform memory access), c’est aussi pour de la performance, pour les pcs avec differents banque de memoire et proprité d’acces, (on ne pas pas aborder la notion de un seul numa node qui disccus avec plusieurs nodes). 

Nous allons rester sur un seul CPU et un seul NUMA node pour ce qui suit. 

les cpu et les nodes ont des pointeurs vers des slabs, mais les nodes utilise une liste doublement chainé


Slab : 
Pour Chaque Slab dans le linux kernel nous avons une structure slab correspondante, et pour chacun il ya une page structure qui elle correspond à une page physique, dépendant que si la page physique est normale est alloué seulement à ce slab, ou si elle est partagé avec un autre, le layout sera différent. 

struct slab { // Aliased with struct page 
struct kmem_cache *slab_cache; // Cache this slab belongs to 
struct slab *next; // Next slab in per-cpu list 
int slabs; // Slabs left in per-cpu list 
struct list_head slab_list; // List 
links in per-node list 
void *freelist; // Per-slab freelist 
…
 }

chaque slab a une backing memory alloué via page_alloc et qui contient les objets slots
struct slab { // Aliased with struct page void *freelist; // Per-slab freelist ... }

freelist pointe vers le premier slot libre dans le slab, le suivant slot libre est pointé par son precedant pour les autres slots libres. 

le freelist pointeur est stocké prés du milieu de l’objet slot c’est pour  To prevent small-sized overflows corrupting freelist pointer. 

le freelist pointer est hashé, le but c’est de rendre difficile de fake un free list pointeur dans les slub exploit, c’est toujours possible mais au lieu de leak l’adresse du freelist on doit aussi leak l’adresse du cache->random, et du swab(ptr_address), ça reste possible mais ça complique la tâche ! mais ce n’est pas la premiere cible/but d’un exploit kernel !

Quand la liste est allouée totalement, le freelist pointeur est juste NULL. 

Plusieurs Slab peuvent être lié entre eux, on peut avoir deux façons :

ici nous donne le pointeur vers le prochain slab, tandis que le int contient le nombre de slabs dans la liste. 
liste simplement chaîné pour per cpu cache
Liste Doublement chaînée pour per node cache

Revenons au cache : 
per cpu slabs sont lié a un cpu en particulier, donc quand un cpu aura besoin d’un objet il regardera d’abord dans cette liste la, c’est seulement une question de performance (locking, CPU caches; etc)


kmem_cache_cpu has one active slab, (réellement le nom c’est CPU Slab pour le slub mode, mais on simplifie le nom pour mieux comprendre, et donc le slab actuellement utilisé on le call “active”)

Un slab active a deux free lists (c’est une notion tres importante pour comprendre pourquoi Slub est la norme aujourd’hui) : 

Free list dans kmem_cache_cpu : 
le but c’est quand le cpu veut un slots pour faire une operation, il n’a pas besoin de parcourir la liste, il va utiliser l’objet directement pointé par Kmem_cache_cpu 

 Free list dans le slab actif (struct page) : 
stocké dans struct page du slab et permet de representer l’etat réel du slab, (l’etat officiel des slots libre dans ce slab). il est utilisé quand un slab est partagé, quand il quitte un cpu, ou quand il passe en partiel. 

Pendant les allocations —> seul le pointeur CPU est modifié (rapide, lockless)
Quand le slab est abandonné ou synchronisé —> la freelist CPU est reversée dans la freelist du slab (struct page)

Les partials Slabs : 
ils ont uniquement une freelist qui est celle de struct page, Used for allocations once active slab gets full.

kmem_cache_node has list of per-node partial slabs : 
Son rôle principal est de gérer une liste de slabs partiellement utilisés (partial slabs), ces ne sont plus attachés à un CPU spécifique, Chaque slab de cette liste possède sa mémoire physique et une seule freelist, stockée dans sa struct page qui décrit tous les objets libres du slab.

Tant que les slabs per-CPU ont des objets libres les allocations se font sans lock, localement, les allocations se font sans lock, localement un slab est pris depuis la liste partial du nœud, ce slab est alors migré vers ce CPU et devient son slab actif.

Limite de Large de ces listes : 

Maximum number of kept per-CPU partial slabs is limited, cette limite se voit ici dans la structure du kmem_cache, le cpu_partial_slabs ne peut pas être directement connu comme valeur, cependant nous avons cpu_partial il est lié à ce calcul dans /sys/kernel/slab/$CACHE/cpu_partial, et donc on peut déduire cpu_partial_slabs. 

connaître ce nombre est intéressant pour certain slab attacks, comme la cross-cache attaque, qui utilise le overflowing de per_cpu_per_partial liste,
En SLUB, le nombre de slabs partiellement utilisés conservés par CPU est volontairement limité afin d’éviter une rétention excessive de mémoire, À l’inverse, SLUB impose un minimum de slabs partiels conservés par nœud NUMA.




voici un schéma qui englobe tout ce qu’on vient de voir : 
 

Slub Internals : Allocation Process : 
il ya 5 tiers de deroulement d’allocation ; 

1- a specific cpu is trying to allocate an object from a specific cache, the first case is per-cpu partial list (lock-list), is empty, si ça n’est pas le cas il retourne le premier objet libre dans la freelist et c’est good ! 

si c’est vide : 

si le lock-less per-CPU freelist est vide, on passe au tier 2, l’allocation depuis le slab freelist (donc avec lock), 



si les deux freelist sont vide, c’est le cas 3, on alloue, le per-cpu partial slabs, le slab allocator prend le premier slab du per-cpu qui a une partial liste et le rend active (on se retrouve dans le cas 1 et 2). 

si on a pas de per_cpu partial list dispo, on fait de l’allocation depuis le per_node, on prend le premier slab dans la slab list et on l’assigne en active, mais en plus de ça nous allons bouger certaines per_node slabs a la per_cpu liste. l’idée derriere c’est vu qu’on a run out de toute les methodes d’allocations (inferieurs et donc moins couteuse en timing et plus performante) ils faut toute les restocker (le slab active avec une free list per cpu et une freelist (chainé) et une partial list). on se retrouvera dans l’etape 1 et 2 ! 

Allocationg from new slab : quand on a aucune slad per_cpu et per_node dans ce cas ; le slab allocateur va juste allouer un nouveau active slab, 


 Allocations happen from active slab : another slab gets assigned as active once free slots in current one run out : 
let's say we just start allocating many many objects from the same cache on the same CPU so we're going to f the active slap we're going to feel all the P CPU partial slabs and all the per not slabs and at some point a new slap is going to be created with all slots empty and it's going to be assigned as the active slab

faire un schéma ou j’explique les étapes de l’allocation
expliquer un objet contient quoi
expliquer NUMA mieux.

