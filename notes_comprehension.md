
Premiere Comprehension du Slab (ces 3 types, et son utilité)

Kernel Constraint : 

Pourquoi le kernel linux a besoin d’une tout autre heap ? 
il doit etre : 
memory efficient 
performant

le glibc est le GNU de linux, ya le userland inclus, et donc ya bcp de meta data (reading, writing), (size, previous size, pointer, nextpointer, large bins)

on perds bcp de temps à bouger ces free chunks entre plusieurs bins, 
pareil si je fais un malloc et que j’ai un chunks plus gros que ce que j’ai besoin le gbc est capable de le diviser en deux et donc peut etre qu’il ya une meilleure implémentation de la heap qui pourra m'éviter ce problème
aussi, le kernel runs forever, c’est pas un programme userland, du coup restarting the kernel c’est essentiellement reboot la machine, donc le kernel doit être memory efficient, il peut pas perdre l’indice en mémoire ou avoir des fuites de mémoires. 

exemple de fragmentation : 
ici il ya de grande chances que le chunks qu’on vient de free sera utilisé pour le second malloc et donc on sépare le grand chunks en deux sous chunks un de 60 et un de 20, si on boucle sur cette opération on aura plusieurs chunks de 20, qui ne serviront a rien, c’est de la fragmentation de mémoire, et c’est quelque chose que le kernel, un programme qui tourne indéfiniment voudrait éviter. 

Slab Allocators : 
dans ce document nous allons utiliser les normes donc slab en minuscule signifie un espace memoire contigue  et SLAB en majuscule signifie une implementation specifique du slab generique. 
il ya 3 types de slab allocators : 
slob : deprecated mais toujours utilisé dans des endroits specifique comme le embeded systems
slab : on peut compiler avec mais ce n’est pas le mode par défaut, 
slub : le mode par defaut aujourd’hui, (uncued slab allocators)

Slab allocator – terminologie kernel
Dans un système d’exploitation, la heap du kernel est différente de la heap du userland, voici la terminologie de la heap du kernel 

caches : gere les objets de taille fixe, un cache contient plusieurs slabs,Exemples :


cache pour objets de 256 octets


cache pour objets de 512 octets
Il est aussi possible de créer des caches personnalisés pour un type précis 


Slabs : c’est un bloc de mémoire contiguë, une ou plusieurs pages physiques libres, un slab est composé de slots (emplacement de taille fixe), un slab appartient toujours à un seul cache. 
slots : est une région mémoire de taille fixe dans le slab, sa taille est fixé par le cache; managé par le cache, donc si le cache est 256 o, les slots seront chacun 256o, un slot peut être : 
libre 
occupé par un objet kernel
exemple : dans le cas d’un kmalloc le pointeur retourné est sur l’un de ces slots disponible, quand il sera utilisé on dira que ce slots contient un objet (un kernel objet)

 schema pwn.college, schema conceptuelle
exemple : 
Page = 4096 bytes
Cache = 256 bytes
Slots par slab = 4096 / 256 = 16


Userland heap
Kernel slab heap
tailles variables
tailles fixes
fragmentation élevée
fragmentation contrôlée
malloc/free
caches + slabs
usage général
objets kernel


**Allocation (kmalloc)**
Déterminer la taille demandée
Choisir le cache correspondant (ex: 256B)
Trouver un slab avec un slot libre
Retirer le slot de la free list
Retourner un pointeur vers ce slot


**Free (kfree)**
Identifier le cache du pointeur
Marquer le slot comme libre
Le remettre dans la free list du slab

**schema plus technique**  

le cache utilise deux types de structures :
- kmemcache cpu 
- kmemecache node
ce qui est a gauche (cpu) peut etre considere comme le “working” slabs in use, 
sur la droite c’est ce qui n’est pas actuellement utilisé par le actuel cpu

pourquoi on a une division ? c’est exactement comme dans le userland avec les threads, on veut pas avoir des threads qui s’attendent entre eux donc, et ca nous permet de savoir ce que le cpu actuelle utilise sans avoir de la contention avec d’autres cpu. 
pour chaque cache pour chaque cpu il ya une active working slab, quand on a besoin d’un new new slab, le kmem_cache va regarder à droite, et claim ownership sur un slab libre, le slab aura surement qlq slots alloué, mais il pourra être utilisé. 
on gardera la trace des slabs avec des slots libre dans une liste nr_partial dans le kmem_cache_node

Comment les objets sont alloués dans le slab, dans les slots et comment ces slots sont reliés ? 

Quand on free un objet, c’est pushed on a single linked list, si on free un second objet, ca fait un push a la tete de la liste et ca change les pointeurs. 
kmem_cache_cpu pointe sur la tete (second object libere) et le second objet pointe sur le suivant (qui lui pointe sur null vu qu’on a seulement deux objets libres). 

