# Bonfire-shell
## The next gen safe shell wrapper

> Created for the bonfire cli firewall

Bonfire shell is a safe wrapper around any cli program
in linux based operating system or one with memfd capabilities.

Both the program and the initial configuration is stored in memory thanks to the memfd
feature in the operating system. This allows a safe way to initialize the program the way
you want, without anyone having access to the program directly!

This means that you can bundle a script with a python interpreter, bash or, well ANYTHING!

### How do I package my app?

In case of python, you change the file name to the /proc/self/fd/{INIT_FD}.
Change the include_str! to your script.
Thats it. Bob's your uncle.

### NOTE!

This is platform specific, probably won't work under windows.
Also having an x86 wrapper around an arm64 program won't work either.
You may also need changes to the source code to make it compatible with your application.

Have fun!
