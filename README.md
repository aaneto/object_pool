# Object Pool

This library implements an Object Pool using the free list pattern: https://gameprogrammingpatterns.com/object-pool.html

This design pattern is useful when you have a pool of objects, and you want to keep track of the free
slots, to achieve that you add a union data type to the pool.

## Implementation

This data type can either be two pointers to the next and previous empty indexes or the actual
data for the Pool, this way you can easily iterate over the linked list to get N free indexes with O(k) performance,
where k is the number of free indexes required.

Looping over the entire array is still O(n), since this is still an array. But keep in mind
that while looping is O(n), n is equal to the number of filled slots + k (empty slots). Meaning we have cache misses
if we iterate doing relevant operations.

## Enhancements

There can be further actions to address this, and other problems. Since this is a POC I did on a Saturday
afternoon, I won't extend this unless I actually need it into a project, this is more of a toy repository.


I don't imagine anyone wants to mess with this, but if you do want, feel free to modify, fork, or do
anything with this code.

### Note to Learners

If you wish to learn more about this pattern and others, I recommend reading about them on the 
website I linked at the very top of the README, reading this code will probably just insert unnecessary details
into your head.
