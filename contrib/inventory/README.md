Inventory scripts for Jetporch
==============================

https://www.jetporch.com/inventory/dynamic-cloud-inventory

About This Repo
===============

Jet uses an inventory script format originally developed for Ansible in 2012, that was in turn inspired by an earlier feature from Puppet
called 'external nodes'.

This directory was started was started with a friendly fork of the ansible inventory scripts. These scripts were removed from the ansible inventory tree after Ansible 2.9
as Ansible moved to a new system of inventory plugins which were not easy to call from other tooling, thus warranting this fork.

With this fork, Jet can support the original inventory script format despite a differences in programming language (Jet is written in Rust). 
We suspect other projects can also make use of this JSON data as is documented on the jetporch homepage, and would encourage this!

What Is Included
================

Take a look at the inventory/ directory for all of the plugins offered.  

Most plugins can work by copying the inventory plugin out of the repo
along with the associated configuration file. Many plugins also respond to environment variables. Details are almost always mentioned
in the source code.

Customizing an inventory script to your own needs is always encouraged if you are so inclined, which is why the included scripts are not part of any 
tightly integrated software package.

Status
======

As the originating code was a handful of years old when we established Jeti (September 2023), a few plugins may need and warrant changes to function, 
or may be in need of additions to surface newer features from some of the services.  Updates via patches are very welcome and patches will be tended to quickly!

We want everything here to function at a high degree of quality, and if things are unsalveagable due to major
API changes, please let us know and these scripts can be retired.

Tested
======

We maintain a list of validated plugins tested so far:

* Azure
* AWS/EC2
* Digital Ocean
* Docker
* GCE
* OpenStack

Nearly all plugins should return JSON, they may have just not been tested
yet with full cloud configurations. Help validating these is welcome.

If you validate that a plugin is working (or has some limitations), let us know with a patch or even just a GitHub ticket.  Plugins that get no users
or tickets after a full year may be deleted.

Support and Development Guidelines
==================================

None of these plugins should be considered unsupported but they should be considered 'community maintained' as not all services therein
or features in those services are ones that the project leadership uses regularly.

Obviously we want inventories for systems such as AWS/EC2, GCE, etc, to function at a particularly high degree of quality.

Please be respectful and do not contact listed authors of older code
for help with forked jeti inventory scripts, names are left around for attribution and copyright reasons, not for support.

Where the plugin imports some code from "jeti.module_utils" in most cases the plugin can be edited to not need these, as much of this
is related to legacy python 2.X support, and we do not mind requiring python3.  Patches to remove imports to jeti.module_utils
(with sufficient testing of the module) will always be accepted! Once all imports from jeti are removed, we can delete
the 'lib/' directory from this repo.

If you would like to port features over from inventory scripts that were made since Ansible 2.9, that is welcome, but make sure
any ports are well tested and have no dependencies on Ansible (or jeti's legacy module_utils directory that we are seeking to remove).
In other words, scripts should be entirely self contained other than external dependencies they have from pypi or possibly cargo for
future Rust dynamic inventory programs.

Note that because Jetporch supports a more liberal interpretation of the historical Ansible module format than Michael developed
in 2012, scripts in this repo (especially new ones) may diverge and not be loadable by classic Ansible. 
We are fine with this and that is not a development concern. This should make modules easier to develop for more new scripts.

See https://www.jetporch.com/inventory/dynamic-cloud-inventory for the JSON specification.

New inventory scripts do not need to accept '--host', as this is never used as Jet does not pass arguments to the inventory script.  Patches
 to remove this feature from any inventory scripts will also be accepted.

While jet is efficient with large datasets, any inventory plugins also return a serious number of variables that may not be deemed usuable for automation purposes, for instance,
if every single host object comes back with the list of possible instance sizes for a region, patches to remove this content are
GREAT candidates. Ideally only theoretically relevant data should be returned in the JSON.

Testing A Script
================

Follow the instructions on any inventory plugin to see if it works with your infrastructure:

1) Chmod +x the plugin, then execute it manually with "./plugin.py" or "python3 ./plugin.py"

2) You may need to install some dependencies, for instance "pip3 install requests" if this fails.  If the plugin does not mention
pip install commands the user needs to run at the top of the file, patches to add them will be welcome! Each plugin can
also have it's own ".md" readme named after the inventory plugin if it wants.

3) You may need to have the configuration file for the module configured.  If so, copy the configure file and the script
out to another directory and edit the configuration file there.  Make sure the changes to configuration files do not
appear in any patches requests back the repo, such as to leak an API key!

4) Now execute the inventory script with jetp: "jetp show --inventory ./plugin.py --groups all" to see how Jet loaded the plugin.
You can now see the same JSON as Jet experiences it!

5) You can now use the inventory plugin with "jetp ssh --inventory ./plugin.py --playbook playbook.yml"!

Authors
=======

Ansible was created by [Michael DeHaan](https://github.com/mpdehaan)
and has contributions from over thousands of users. We are thankful for every one
of their contributions here. [Ansible](https://www.ansible.com) is maintained by and
is a trademark of [Red Hat, Inc](https://www.redhat.com>)

Contributions to this repo from September 2023 and beyond are made by
contributors of Jeti, a subproject of [Jetporch](https://www.jetporch.com)

License
=======

GNU General Public License v3.0 or later, see COPYING for details.


