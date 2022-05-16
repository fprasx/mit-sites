# mit-sites

Finding sites ending in .mit.edu. Why? uhh

Run `cat out/* sites.txt | sort -u -o sites.txt` to aggregate all sites found
from `sites.txt` and `out/*` into sites.txt. There could be some sites that only
appeared in some runs or some that were manually added to `sites.txt`.

## Depth-first Search

We go to a webpage, finding all the links on that page. Then, for each link on
that page, we repeat the process of finding all the links on that page and
descending one level. We do this with a recursive function. If we have already
seen a link, we skip it and go onto the next link to avoid doing the same work
multiple times.

## CLI options for `depth_first_search.py`
```
usage: depth_first_search.py [-h] [-d DEPTH] [--debug] [--no-save]

Process arguments for depth-first search

optional arguments:
  -h, --help            show this help message and exit
  -d DEPTH, --depth DEPTH
                        how deep the search goes, note, larger values will take longer
  --debug               print debug output
  --no-save             don't save output to file
```

## Dependencies
* requests_html
