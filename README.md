# do-ssh
Tunnels a ssh-connection over iroh.

## How to use this on the client?
Use the compiled executable as a ProxyCommand for ssh.
```ssh USER@NODEID -o "ProxyCommand=do-ssh %h"```
You can use this with password login or key login, it supports everything native ssh supports.
This program is just a proxy that tunnels the ssh-connection over iroh.
Iroh uses direct connections over QUIC, which are End-to-End encrypted, so there is no risk on typing your password or using your key.

Tip: Use the ssh-config file to specify the ProxyCommand:
```
Host EXAMPLE
  Hostname NODEID
  ProxyCommand do-ssh %h
  # IdentityFile ~/.ssh/id_rsa
```

## How to use this in the server?
`do-ssh`.
That's it.
This generates a persistant SecretKey at the current working directory and starts an iroh endpoint to connect to.
Use the printed NodeId as the hostname for the ssh command and you are good to go.

Tip: Use something like a systemd-service to start the proxy on boot.

## Why?
I got a raspberry pi laying around and recently started a project. But the pi often is at a different location and behind a NAT.
To get around this I discovered iroh to create direct connections between devices. So I created this...

## Problems
The delay is notable especially with poor connections.
If the IP of the client or server changes, the connection will most likely rebuild itself automatically, but this may take some seconds.

If a direct connection cannot be established iroh uses its relay servers to transport packages.
Remember, the connection is End-to-End encrypted but I never looked deeply into the source code of iroh.
If you want to be absolutely sure, check out the imo awesome iroh project [here](https://github.com/n0-computer/iroh).

Because the proxy runs on the server itself the only way to log connections (with real addresses) is only possible from the proxy itself (ssh will always show 127.0.0.1 because this is where the TCP-connection comes from), which currently is not implemented.

### TODO: Planned in future:
- Provide compiled releases.
- Code-Documentation.
- Support for specifying port on server side.
- Log connections (print NodeId of client).

REMEMBER: This is more a proof of concept. It works (im using it myself) but NodeIds of clients change and the implemented iroh-protocol may receive breaking changes in the future.

~doEggi
