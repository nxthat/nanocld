# Usefull command

# lsns - list namespaces

```sh
lsns
```

# nsenter - run program in different namespaces

```sh
sudo nsenter -t 12267 -n ss -ltumc
```
