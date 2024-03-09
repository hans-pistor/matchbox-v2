To create a code drive, we can run the following


```
truncate -s 1G /tmp/code-drive.img
mkfs.ext4 /tmp/code-drive.img
mount /tmp/code-drive.img /tmp/code
```

Edit /tmp/code/entrypoint.py to be whatever you want to be executed