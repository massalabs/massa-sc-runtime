# A script to extract wasm simd operator from spec
# Warning: try to translate op name from wasm spec to wasmer op name (but this is not accurate)

import tempfile
import urllib.request
from itertools import islice
import collections
import re
import string


def consume(iterator, n=None):
    """ Advance the iterator n-steps ahead. If n is None, consume entirely.
    """
    # Use functions that consume iterators at C speed.
    if n is None:
        # feed the entire iterator into a zero-length deque
        collections.deque(iterator, maxlen=0)
    else:
        # advance to the empty slice starting at position n
        next(islice(iterator, n, n), None)


def extract_operator_from_spec_file(filepath):
    """Return operator + operator name (as wasmer 2.3 defined it)"""

    rgx = re.compile(r"`([\w._]+)`")
    with open(filepath) as fp:
        consume(fp, 2)
        for line in fp:
            # print(line)
            rgx_res = rgx.search(line)
            if rgx_res:
                op = rgx_res.group(1)  # remove `at the start & at the end
                wasmer_op_name = "".join([string.capwords(i) for i in re.split(r"\.|_", op)])
                # print(f'"{wasmer_op_name}",')
                yield op, wasmer_op_name


def main():

    url_simd_spec = "https://raw.githubusercontent.com/WebAssembly/simd/main/proposals/simd/ImplementationStatus.md"

    g = urllib.request.urlopen(url_simd_spec)
    # with open('test.png', 'b+w') as f:
    with tempfile.NamedTemporaryFile() as fp:
        fp.write(g.read())
        for op, wasmer_op_name in extract_operator_from_spec_file(fp.name):
            # print(op, " -- ", wasmer_op_name)
            print(f'"{wasmer_op_name}",')


if __name__ == "__main__":

    main()
