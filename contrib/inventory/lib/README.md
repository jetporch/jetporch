Note for developers browsing this folder:

All files in this directory are files we do not want inventory scripts to use but are included because
some scripts may still import them, which is fine really. Most of it deals with Python 2.X compatibility.

Generally we want inventory scripts to only rely on modules from pypi (pip3 install ...) and be self-contained.
Requiring python3 is totally acceptable especially since we do not require python to be installed on remote hosts.

Since we can assume python3 support is mainstream, removal of imports using six, etc, can be taken
out of files in inventory/ any time and pull requests for this are quite welcome!  

When no more imports of jeti.* are included in the inventory/ directory scripts, we can completely delete this
directory, which is our (not very urgent) goal.

No new code should go into this directory.

--Jetporch project

