# mit-sites

Finding sites ending in .mit.edu. Why? uhh

Well, we're at **227** so far.

# Method

We use Depth-first search and Breadth-first search (TODO).

## DFS

We got to a webpage, finding all the links on that page. Then, for each link on that page, we repeat the process of finding all the links on that page and descending one level. We do this with a recursive function. If we have already seen a link, we skip it and go onto the next link to avoid doing the same work multiple times.

## BFS

TODO

TODO: sort out giving and alum thing, case when there is only one link on the site and it has already been found (should be sorted out by base case don't know why it's not working)
