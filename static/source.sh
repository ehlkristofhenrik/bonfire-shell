FIRST_RUN=1

function firewall() {
    if [ $FIRST_RUN -eq 1 ];then
        FIRST_RUN=0
        return
    fi

    ./bonfire-client "$BASH_COMMAND"
    if [[ $? -ne 0 ]]; then
        echo "You shall not pass!"
        exit 1
    fi
}

readonly -f firewall

enable -n readonly
trap firewall DEBUG
enable -n trap enable
