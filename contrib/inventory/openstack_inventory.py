#!/usr/bin/env python3

# Copyright (c) 2012, Marco Vito Moscaritolo <marco@agavee.com>
# Copyright (c) 2013, Jesse Keating <jesse.keating@rackspace.com>
# Copyright (c) 2015, Hewlett-Packard Development Company, L.P.
# Copyright (c) 2016, Rackspace Australia
#
# This module is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# This software is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with this software.  If not, see <http://www.gnu.org/licenses/>.

# The OpenStack Inventory module uses os-client-config for configuration.
# https://github.com/openstack/os-client-config
# It also depends on the python-openstackclient library.
# https://docs.openstack.org/python-openstackclient/latest/
# This means it will either:
#  - Respect normal OS_* environment variables like other OpenStack tools
#  - Read values from a clouds.yaml file.
# If you want to configure via clouds.yaml, you can put the file in:
#  - Current directory
#  - ~/.config/openstack/clouds.yaml
#  - /etc/openstack/clouds.yaml
#  - /etc/jeti/clouds.yaml
# The clouds.yaml file can contain entries for multiple clouds and multiple
# regions of those clouds. If it does, this inventory module will by default
# connect to all of them and present them as one contiguous inventory.  You
# can limit to one cloud by passing the `--cloud` parameter, or use the
# OS_CLOUD environment variable.  If caching is enabled, and a cloud is
# selected, then per-cloud cache folders will be used.
#
# See the adjacent openstack.yml file for an example config file
# There are two jeti inventory specific options that can be set in
# the inventory section.
# expand_hostvars controls whether or not the inventory will make extra API
#                 calls to fill out additional information about each server
# fail_on_errors causes the inventory to fail and return no hosts if one cloud
#                has failed (for example, bad credentials or being offline).
#                When set to False, the inventory will return hosts from
#                whichever other clouds it can contact. (Default: True)
#

import argparse
import collections
import os
import sys
import time
from packaging.version import Version
from io import StringIO

import json

import openstack as sdk
from openstack.cloud import inventory as sdk_inventory
from openstack.config import loader as cloud_config

CONFIG_FILES = ['/etc/jeti/openstack.yaml', '/etc/jeti/openstack.yml']


def get_groups_from_server(server_vars):

    groups = []

    # set cloud from server vars, if it exists
    # it can exist in 2 possible locations, depending on the version of openstacksdk
    # it can either be under 'cloud' or under 'location/cloud'
    cloud = server_vars.get('cloud', None)
    if cloud is None:
        cloud = server_vars.get('location', {}).get('cloud', None)
    if cloud:
        groups.append(cloud)

    # set region from server vars, if it exists
    # it can exist in 2 possible locations, depending on the version of openstacksdk
    # it can either be under 'region' or under 'location/region_name'
    region = server_vars.get('region', None)
    if region is None:
        region = server_vars.get('location', {}).get('region_name', None)
    if region:
        groups.append(region)

    # set group from server vars, if it exists
    metadata = server_vars.get('metadata', {})

    # And one by cloud_region
    if cloud and region:
        groups.append("%s_%s" % (cloud, region))

    # Check if group metadata key in servers' metadata
    if 'group' in metadata:
        groups.append(metadata['group'])

    for extra_group in metadata.get('groups', '').split(','):
        if extra_group:
            groups.append(extra_group.strip())

    # groups.append('instance-%s' % server_vars['id'])
    if 'name' in server_vars and server_vars['name'] != '':
        groups.append(server_vars['name'])

    for key in ('flavor', 'image'):
        if 'name' in server_vars[key]:
            groups.append('%s-%s' % (key, server_vars[key]['name']))

    # for key, value in iter(metadata.items()):
    #     groups.append('meta-%s_%s' % (key, value))

    az = server_vars.get('az', None)
    if az:
        # Make groups for az, region_az and cloud_region_az
        groups.append(az)
        groups.append('%s_%s' % (region, az))
        groups.append('%s_%s_%s' % (cloud, region, az))

    return groups


def get_host_groups(inventory, refresh=False, cloud=None):

    (cache_file, cache_expiration_time) = get_cache_settings(cloud)
    if is_cache_stale(cache_file, cache_expiration_time, refresh=refresh):
        groups = to_json(get_host_groups_from_cloud(inventory))
        with open(cache_file, 'w') as f:
            f.write(groups)
    else:
        with open(cache_file, 'r') as f:
            groups = f.read()
    return groups


def append_hostvars(hostvars, groups, key, server):

    hostvars[key] = server

    # make sure all hosts are added to a 'hosts' sub-group
    for group in get_groups_from_server(server):
        if group not in groups:
            groups[group] = dict(hosts=[])
        groups[group]['hosts'].append(key)


def get_host_groups_from_cloud(inventory):

    firstpass = collections.defaultdict(list)
    groups = collections.defaultdict(list)
    hostvars = {}
    list_args = {}

    # set expand_hostvars and fail_on_errors from inventory.extra_config, if available
    if hasattr(inventory, 'extra_config'):
        list_args['expand'] = inventory.extra_config['expand_hostvars']
        if Version(sdk.version.__version__) >= Version("0.13.0"):
            list_args['fail_on_cloud_config'] = \
                inventory.extra_config['fail_on_errors']

    # add hosts to firstpass grouped together by hostname
    for server in inventory.list_hosts(**list_args):
        if 'interface_ip' not in server:
            continue
        firstpass[server['name']].append(server)

    for name, servers in firstpass.items():
        for server in servers:
            append_hostvars(hostvars, groups, name, server)

    groups['all'] = {'hostvars': hostvars}

    return groups


def is_cache_stale(cache_file, cache_expiration_time, refresh=False):
    ''' Determines if cache file has expired, or if it is still valid '''
    if refresh:
        return True
    if os.path.isfile(cache_file) and os.path.getsize(cache_file) > 0:
        mod_time = os.path.getmtime(cache_file)
        current_time = time.time()
        if (mod_time + cache_expiration_time) > current_time:
            return False
    return True


def get_cache_settings(cloud=None):
    config_files = cloud_config.CONFIG_FILES + CONFIG_FILES
    if cloud:
        config = cloud_config.OpenStackConfig(
            config_files=config_files).get_one(cloud=cloud)
    else:
        config = cloud_config.OpenStackConfig(
            config_files=config_files).get_all()[0]
    # For inventory-wide caching
    cache_expiration_time = config.get_cache_expiration_time()
    cache_path = config.get_cache_path()
    if cloud:
        cache_path = '{0}_{1}'.format(cache_path, cloud)
    if not os.path.exists(cache_path):
        os.makedirs(cache_path)
    cache_file = os.path.join(cache_path, 'jeti-inventory.cache')
    return (cache_file, cache_expiration_time)


def to_json(in_dict):
    return json.dumps(in_dict, sort_keys=True, indent=2)


def parse_args():
    parser = argparse.ArgumentParser(description='OpenStack Inventory Module')
    parser.add_argument('--cloud',
                        default=os.environ.get('OS_CLOUD'),
                        help='Cloud name (default: None')
    parser.add_argument('--private',
                        default=os.environ.get('OS_PRIVATE', False),
                        action='store_true',
                        help='Use private address for jeti host')
    parser.add_argument('--refresh',
                        default=os.environ.get('OS_REFRESH', False),
                        action='store_true',
                        help='Refresh cached information')
    parser.add_argument('--debug',
                        default=os.environ.get('OS_DEBUG', False),
                        action='store_true',
                        help='Enable debug output')

    return parser.parse_args()


def main():

    args = parse_args()

    try:
        # openstacksdk library may write to stdout, so redirect this
        sys.stdout = StringIO()
        config_files = cloud_config.CONFIG_FILES + CONFIG_FILES
        sdk.enable_logging(debug=args.debug)

        inventory_args = dict(
            refresh=args.refresh,
            config_files=config_files,
            private=args.private,
            cloud=args.cloud,
        )

        if hasattr(sdk_inventory.OpenStackInventory, 'extra_config'):
            inventory_args.update(dict(
                config_key='jeti',
                config_defaults={
                    'use_hostnames': True,
                    'expand_hostvars': True,
                    'fail_on_errors': True,
                }
            ))

        inventory = sdk_inventory.OpenStackInventory(**inventory_args)

        sys.stdout = sys.__stdout__
        # always return list of items; individual hosts are not supported
        output = get_host_groups(inventory, refresh=args.refresh, cloud=args.cloud)
        print(output)

    except sdk.exceptions.OpenStackCloudException as e:
        sys.stderr.write('%s\n' % e.message)
        sys.exit(1)

    sys.exit(0)


if __name__ == '__main__':
    main()
