# Direct port of https://github.com/wbingli/zsh-wakatime to Nushell

def waka-projectname [] {
    if ($env.NU_WAKATIME_PROJECT_DETECTION? | default "false") == "true" {
        if (which git | is-not-empty) {
            try {
                git config --local remote.origin.url
                | parse --regex '.*/(?<name>[^/.]+)(\.git)?$'
                | get 0.name
            } catch {
                "<<LAST_PROJECT>>"
            }
        } else {
            "<<LAST_PROJECT>>"
        }
    } else {
        "Terminal"
    }
}

def waka-entity [] {
    if ($env.WAKATIME_USE_DIRNAME? | default "false") == "true" {
        $env.PWD
    } else {
        # only command without arguments to avoid senstive information
        # if a filename, 
        history | last | get --optional command | default "" | parse --regex '^\s*(?<cmd>\S+)' | get cmd | first | default ""
    }
}

def send-wakatime-heartbeat [] {
    let entity = (waka-entity)

    if ($entity | is-not-empty) {
        let project = (waka-projectname)

        # ^wakatime-cli --write --plugin "nushell-wakatime/0.0.1" --entity-type app --project $project --entity $entity | ignore
        # temporary
        echo $"Sending WakaTime heartbeat: project='(project)' entity='(entity)'"
    }
}

$env.config = (
    $env.config | upsert hooks {
        pre_prompt: [
            {||
                send-wakatime-heartbeat
            }
        ]
    }
)