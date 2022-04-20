from typing import Set
from requests_html import HTMLSession
from urllib.parse import urlsplit
import pprint

MAX_DEPTH = 8

def get_base_path(link: str) -> str:
    '''
    Return the site name.

    Ex. https://web.mit.edu/blah/bloh
    becomes https://web.mit.edu
    '''
    return 'https://' + urlsplit(link).netloc

def scan(prev_link: str, prev_found: Set[str], depth: int, session: HTMLSession) -> Set[str]:
    # End recursion
    if depth == MAX_DEPTH:
        return prev_found
    new_links = set()
    res = session.get(prev_link)
    # All .mit.edu links on the page
    links = filter(lambda l: l[-8:] == '.mit.edu', map(get_base_path, res.html.absolute_links))
    for link in links:
        # Skip it if we've already searched this link
        if link == prev_link:
            continue
        # We only want to search if we haven't searched the link before
        if not link in new_links:
            new_links.add(link)
            print(f'{"    " * depth}Searching link {link} originating from {prev_link} at depth {depth}')
            new_links = new_links.union(scan(link, new_links, depth + 1, session)) 
    return new_links

if __name__ == '__main__':
    root_url = "https://eecs.mit.edu"
    session = HTMLSession()

    links = set()
    s = scan(root_url, links, 0, session)
    pprint.pprint(s)
    # res = session.get(root_url)
    # a = res.html.absolute_links
    # for link in a:
    #     bp = get_base_path(link)
    #     if bp[-8:] == ".mit.edu":
    #         links.add(bp)

    # pprint.pprint(links)
