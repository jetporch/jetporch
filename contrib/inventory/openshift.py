#!/usr/bin/env python3

# (c) 2013, Michael Scherer <misc@zarb.org>
#
# This file is part of jeti, which was forked from Ansible
#
# Jeti is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# Jeti is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with Jeti.  If not, see <http://www.gnu.org/licenses/>.

DOCUMENTATION = '''
---
inventory: openshift
short_description: Openshift gears external inventory script
description:
  - Generates inventory of Openshift gears using the REST interface
  - this permit to reuse playbook to setup an Openshift gear
version_added: None
author: Michael Scherer
'''

import json
import os
import os.path
import sys
from io import StringIO
import requests
from requests.auth import HTTPBasicAuth

import configparser as ConfigParser

configparser = None


def get_from_rhc_config(variable):
    global configparser
    CONF_FILE = os.path.expanduser('~/.openshift/express.conf')
    if os.path.exists(CONF_FILE):
        if not configparser:
            ini_str = '[root]\n' + open(CONF_FILE, 'r').read()
            configparser = ConfigParser.ConfigParser()
            configparser.readfp(StringIO.StringIO(ini_str))
        try:
            return configparser.get('root', variable)
        except ConfigParser.NoOptionError:
            return None


def get_config(env_var, config_var):
    result = os.getenv(env_var)
    if not result:
        result = get_from_rhc_config(config_var)
    if not result:
        sys.exit("failed=True msg='missing %s'" % env_var)
    return result


def get_json_from_api(url, username, password):
    headers = {'Accept': 'application/json; version=1.5'}
    auth = HTTPBasicAuth(username, password)
    response = requests.get(url, headers=headers, auth=auth)
    return response.json['data']


username = get_config('JETI_OPENSHIFT_USERNAME', 'default_rhlogin')
password = get_config('JETI_OPENSHIFT_PASSWORD', 'password')
broker_url = 'https://%s/broker/rest/' % get_config('JETI_OPENSHIFT_BROKER', 'libra_server')


response = get_json_from_api(broker_url + '/domains', username, password)

response = get_json_from_api("%s/domains/%s/applications" %
                             (broker_url, response[0]['id']), username, password)

result = {}
for app in response:

    # ssh://520311404832ce3e570000ff@blog-johndoe.example.org
    (user, host) = app['ssh_url'][6:].split('@')
    app_name = host.split('-')[0]

    result[app_name] = {}
    result[app_name]['hosts'] = []
    result[app_name]['hosts'].append(host)
    result[app_name]['vars'] = {}
    result[app_name]['vars']['jet_ssh_user'] = user

if len(sys.argv) == 2 and sys.argv[1] == '--list':
    print(json.dumps(result))
elif len(sys.argv) == 3 and sys.argv[1] == '--host':
    print(json.dumps({}))
else:
    print("Need an argument, either --list or --host <host>")
