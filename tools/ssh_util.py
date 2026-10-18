#!/usr/bin/env python3
"""
ssh_util.py - Centralized SSH Connection and Authentication Utility for Longan Nano Test Tools.

Complies with [MON-SEC-01]:
- Loads host, user, password, and SSH key path from environment variables:
    LONGAN_SSH_HOST (default: "192.168.0.63")
    LONGAN_SSH_USER (default: "arun")
    LONGAN_SSH_PASS (default: "arun")
    LONGAN_SSH_KEY  (optional path to private key)
- Replaces unconditional AutoAddPolicy with configurable host key policies:
    LONGAN_SSH_STRICT_HOST_CHECK=1 -> paramiko.RejectPolicy()
    LONGAN_SSH_WARN_HOST_KEY=1     -> paramiko.WarningPolicy()
    Default                        -> paramiko.AutoAddPolicy() with loaded system host keys
- Automatically supports SSH key authentication, agent authentication, and password fallback.
"""

import os
import sys
import paramiko

HOST = os.environ.get("LONGAN_SSH_HOST", "192.168.0.63")
USER = os.environ.get("LONGAN_SSH_USER", "arun")
PASS = os.environ.get("LONGAN_SSH_PASS", "arun")
KEY_PATH = os.environ.get("LONGAN_SSH_KEY", None)
STRICT_HOST_CHECK = os.environ.get("LONGAN_SSH_STRICT_HOST_CHECK", "0").lower() in ("1", "true", "yes")
WARN_HOST_KEY = os.environ.get("LONGAN_SSH_WARN_HOST_KEY", "0").lower() in ("1", "true", "yes")

def get_host_key_policy():
    if STRICT_HOST_CHECK:
        return paramiko.RejectPolicy()
    elif WARN_HOST_KEY:
        return paramiko.WarningPolicy()
    else:
        return paramiko.AutoAddPolicy()

def create_ssh_client(host=None, user=None, password=None, key_filename=None, timeout=15, banner_timeout=30):
    """
    Initializes and connects a paramiko.SSHClient using configured environment parameters.
    """
    client = paramiko.SSHClient()
    client.load_system_host_keys()
    client.set_missing_host_key_policy(get_host_key_policy())

    target_host = host or HOST
    target_user = user or USER
    target_pass = password if password is not None else PASS
    target_key = key_filename or KEY_PATH

    connect_kwargs = {
        "hostname": target_host,
        "username": target_user,
        "timeout": timeout,
        "banner_timeout": banner_timeout,
        "look_for_keys": True,
        "allow_agent": True,
    }

    if target_key and os.path.exists(target_key):
        connect_kwargs["key_filename"] = target_key
    elif target_pass:
        connect_kwargs["password"] = target_pass

    client.connect(**connect_kwargs)
    return client
