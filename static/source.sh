function firewall() {
    ls
    if [ $? -ne 0 ]; then
        echo "You shall not pass!"
        exit
    fi
}

readonly -f firewall

enable -n readonly
trap firewall DEBUG
enable -n trap enable
