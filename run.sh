#!/usr/bin/env bash

# Environment
if [ -f "/etc/os-release" ]; then
    source /etc/os-release
    os=$ID
else
    os=$(uname)
fi
topdir=$PWD

# Options
help_message=$(cat <<-END
    Usage:
        run.sh -h
            Print this help message
        run.sh -a [action]
            Perform an action
        run.sh
            Perform an action interactively
    Available actions:
        build
            Build everything
        build-cpython
            Configure and build CPython
        build-kclvm
            Package CPython and the KCLVM extension into KCLVM
        update-kclvm
            Quickly update KCLVM without packaging CPython and site-packages
        test
            Perform testing
        release
            Create a package for releasing
END
)
action=
while getopts "a:h:s:" opt; do
    case $opt in
        a)
        action="$OPTARG"
        ;;
        h)
        echo "$help_message"
        exit 1
        ;;
        s)
        sslpath="$OPTARG"
        ;;
        \?) echo "Invalid option -$OPTARG"
        ;;
    esac
done

if [ "$action" == "" ]; then
    PS3='Please select the action: '
    options=("build" "build-kclvm" "update-kclvm" "test" "release")
    select action in "${options[@]}"
    do
        case $action in
            "build")
                break
                ;;
            "build-kclvm")
                break
                ;;
            "update-kclvm")
                break
                ;;
            "test")
                break
                ;;
            "release")
                break
                ;;
            *) echo "Invalid action $REPLY:$action"
                exit 1
                break
                ;;
        esac
    done
fi

os=$os topdir=$topdir sslpath=$sslpath $topdir/internal/scripts/$action.sh
