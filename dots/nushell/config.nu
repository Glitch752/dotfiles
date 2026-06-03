source './aliases.nu'

use std/dirs

$env.config.buffer_editor = ["code", "-w"]

# Theming
$env.config.table.mode = 'single'
$env.config.show_banner = false
source './theme.nu'

use "oh-my.nu" git_prompt
$env.PROMPT_COMMAND = { (git_prompt).left_prompt }
$env.PROMPT_COMMAND_RIGHT = { (git_prompt).right_prompt }
$env.PROMPT_INDICATOR = " "

# If `gstat` doesn't exist, use the plugin
if ((which gstat | length) == 0) {
    plugin add "nu_plugin_gstat"
    plugin use gstat
}

fastfetch