# mit-sites

Finding sites ending in .mit.edu. Why? uhh

Well, we're at **252** so far.

Run `cat out/* sites.txt | sort -u -o sites.txt ` to aggregate all sites found
into `sites.txt` (There could bes some sites added manually, so we can't just use the results of the searches)

# Method

We use Depth-first search and Breadth-first search (TODO).

## DFS

We go to a webpage, finding all the links on that page. Then, for each link on
that page, we repeat the process of finding all the links on that page and
descending one level. We do this with a recursive function. If we have already
seen a link, we skip it and go onto the next link to avoid doing the same work
multiple times.

## BFS

TODO
