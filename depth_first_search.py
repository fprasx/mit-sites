from typing import Set
from requests_html import HTMLSession
from urllib.parse import urlsplit
from time import time as unix_time
import argparse

# CLI Args
MAX_DEPTH = 10
DBG = False
SAVE = True


def get_id(link: str) -> str:
    '''
    Return the site name without https:// in front and without the trailing .mit.edu

    Ex. https://web.mit.edu/blah/bloh
    becomes web
    '''
    return urlsplit(link).netloc[:-8]

def get_base_url(link: str) -> str:
    '''
    Return the site name.

    Ex. https://web.mit.edu/blah/bloh
    becomes https://web.mit.edu
    '''
    return 'https://' + urlsplit(link).netloc

def get_links(url: str, session: HTMLSession) -> Set[str]:
    # Returns error if connection isn't secure because we are using https
    try: 
        res = session.get(url)
    except Exception as e:
        if DBG:
            print(f"{url} produced the following exception:\n{e}")
        return set()
    # All .mit.edu links on the page
    links = filter(lambda l: l[-8:] == '.mit.edu', map(get_base_url, res.html.absolute_links))
    return set(links)

def scan(prev_link: str, prev_found: Set[str], depth: int, session: HTMLSession) -> Set[str]:
    links = get_links(prev_link, session)
    if depth == MAX_DEPTH:
        prev_found = prev_found.union(links)
        return prev_found 
    for link in links:
        # Skip it if it's a link pointing to the same webpage
        # eg. web.mit.edu pointing to web.mit.edu/blah
        if link == prev_link:
            continue
        # We only want to search if we haven't searched the link before
        if not link in prev_found:
            prev_found.add(link)
            if DBG:
                print(f'{"  " * depth}<{get_id(link)}>  @ ({depth})')
            prev_found = prev_found.union(scan(link, prev_found, depth + 1, session) )
    return prev_found

if __name__ == '__main__':
    # CLI stuff
    parser = argparse.ArgumentParser(description='Process arguments for depth-first search')
    parser.add_argument('-d', '--depth', type=int, help='how deep the search goes, note, larger values will take longer')
    parser.add_argument('--debug', action='store_true', help='print debug output')
    parser.add_argument('--no-save', action='store_true', help='don\'t save output to file')
    args = parser.parse_args()
    MAX_DEPTH = args.depth
    SAVE = args.no_save
    DBG = args.debug
    root = 'https://people.csail.mit.edu'
    session = HTMLSession()
    # Retrieve links that have already been found
    found = set()
    roots = ["https://libraries.mit.edu"]
    # with open('sites.txt', 'r') as file:
    #     for i in file.readlines():
    #         found.add('https://' + i.strip() + '.mit.edu')
    #         # Use all previously found sites as roots
    #         roots.append('https://' + i.strip() + '.mit.edu')
    for root in roots:
        s = scan(root, found, 0, session)
        found = found.union(s)
    with open(f'out/depth-{MAX_DEPTH}-{round(unix_time())}.txt', 'x') as f:
        for i in s:
            f.write(f'{get_id(i)}\n')
    for i in s:
        print(get_id(i))