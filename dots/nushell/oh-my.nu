# oh-my.nu modified for prompts I like

# REQUIREMENTS #
# you definitely need nerd fonts https://www.nerdfonts.com
# nerd fonts repo https://github.com/ryanoasis/nerd-fonts
# i use "FiraCode Nerd Font Mono" on mac
#
# you also must have the gstat plugin installed and registered

# ATTRIBUTION #
# A little fancier prompt with git information
# inspired by https://github.com/xcambar/purs
# inspired by https://github.com/IlanCosman/tide
# inspired by https://github.com/JanDeDobbeleer/oh-my-posh

# Abbreviate home path
def home_abbrev [os_name] {
  let is_home_in_path = ($env.PWD | str starts-with $nu.home-dir)
  if $is_home_in_path {
    if ($os_name =~ "windows") {
      let home = ($nu.home-dir | str replace -ar '\\' '/')
      let pwd = ($env.PWD | str replace -ar '\\' '/')
      $pwd | str replace $home '~'
    } else {
      $env.PWD | str replace $nu.home-dir '~'
    }
  } else {
    if ($os_name =~ "windows") {
      # remove the C: from the path
      $env.PWD | str replace -ar '\\' '/' | str substring 2..
    } else {
      $env.PWD
    }
  }
}

def path_abbrev_if_needed [apath term_width] {
  # probably shouldn't do coloring here but since we're coloring
  # only certain parts, it's kind of tricky to do it in another place
  let T = (ansi { fg: "#BCBCBC" bg: "#3465A4" }) # truncated
  let P = (ansi { fg: "#E4E4E4" bg: "#3465A4" }) # path
  let PB = (ansi { fg: "#E4E4E4" bg: "#3465A4" attr: b }) # path bold
  let R = (ansi reset)
  let is_home_in_path = ($env.PWD | str starts-with $nu.home-dir)

  if (($apath | str length) > ($term_width / 2)) {
    # split out by path separator into tokens
    # don't use psep here because in home_abbrev we're making them all '/'
    let splits = ($apath | split row '/')

    let splits_len = ($splits | length)
    # get all the tokens except the last
    let tokens = (
      1..<($splits_len - 1) | each {|x|
        $"($T)((($splits) | get $x | split chars) | get 0)($R)"
      }
    )

    # need an insert command
    let tokens = ($tokens | prepend $"($T)~")

    # append the last part of the path
    let tokens = ($tokens | append $"($PB)($splits | last)($R)")

    # collect
    $tokens | str join $"($T)/"
  } else {
    $"($P)($apath)($R)"
  }
}

def get_index_change_count [gs] {
  let index_new = ($gs | get idx_added_staged)
  let index_modified = ($gs | get idx_modified_staged)
  let index_deleted = ($gs | get idx_deleted_staged)
  let index_renamed = ($gs | get idx_renamed)
  let index_typechanged = ($gs | get idx_type_changed)

  $index_new + $index_modified + $index_deleted + $index_renamed + $index_typechanged
}

def get_working_tree_count [gs] {
  let wt_modified = ($gs | get wt_modified)
  let wt_deleted = ($gs | get wt_deleted)
  let wt_typechanged = ($gs | get wt_type_changed)
  let wt_renamed = ($gs | get wt_renamed)

  $wt_modified + $wt_deleted + $wt_typechanged + $wt_renamed
}

def get_conflicted_count [gs] {
  ($gs | get conflicts)
}

def get_untracked_count [gs] {
  ($gs | get wt_untracked)
}

def get_branch_name [gs] {
  let br = ($gs | get branch)
  if $br == "no_branch" {
    ""
  } else {
    $br
  }
}

def get_ahead_count [gs] {
  ($gs | get ahead)
}

def get_behind_count [gs] {
  ($gs | get behind)
}

def get_icons_list [] {
  {
    AHEAD_ICON: (char branch_ahead) # "↑" 2191
    BEHIND_ICON: (char branch_behind) # "↓" 2193
    NO_CHANGE_ICON: (char branch_identical) # ≣ 2263
    HAS_CHANGE_ICON: "*"
    INDEX_CHANGE_ICON: "♦"
    WT_CHANGE_ICON: "✚"
    CONFLICTED_CHANGE_ICON: "✖"
    UNTRACKED_CHANGE_ICON: (char branch_untracked) # ≢ 2262
    INSERT_SYMBOL_ICON: "❯"
    HAMBURGER_ICON: (char hamburger) # "≡" 2261
    GITHUB_ICON: "" # f408
    BRANCH_ICON: (char nf_branch) # "" e0a0
    REBASE_ICON: "" # e728
    TAG_ICON: "" # f412
  }
}

def get_icon_by_name [name] {
  get_icons_list | get $name
}

# ╭─────────────────────┬───────────────╮
# │ idx_added_staged    │ 0             │ #INDEX_NEW
# │ idx_modified_staged │ 0             │ #INDEX_MODIFIED
# │ idx_deleted_staged  │ 0             │ #INDEX_DELETED
# │ idx_renamed         │ 0             │ #INDEX_RENAMED
# │ idx_type_changed    │ 0             │ #INDEX_TYPECHANGE
# │ wt_untracked        │ 0             │ #WT_NEW
# │ wt_modified         │ 0             │ #WT_MODIFIED
# │ wt_deleted          │ 0             │ #WT_DELETED
# │ wt_type_changed     │ 0             │ #WT_TYPECHANGE
# │ wt_renamed          │ 0             │ #WT_RENAMED
# │ ignored             │ 0             │
# │ conflicts           │ 0             │ #CONFLICTED
# │ ahead               │ 0             │
# │ behind              │ 0             │
# │ stashes             │ 0             │
# │ repo_name           │ nushell       │
# │ tag                 │ no_tag        │
# │ branch              │ main          │
# │ remote              │ upstream/main │
# ╰─────────────────────┴───────────────╯

def get_repo_status [gs os] {
  let display_path = (path_abbrev_if_needed (home_abbrev $os.name) (term size).columns)
  let branch_name = (get_branch_name $gs)
  let ahead_cnt = (get_ahead_count $gs)
  let behind_cnt = (get_behind_count $gs)
  let index_change_cnt = (get_index_change_count $gs)
  let wt_change_cnt = (get_working_tree_count $gs)
  let conflicted_cnt = (get_conflicted_count $gs)
  let untracked_cnt = (get_untracked_count $gs)
  let has_no_changes = (
    if ($index_change_cnt <= 0) and
    ($wt_change_cnt <= 0) and
    ($conflicted_cnt <= 0) and
    ($untracked_cnt <= 0) {
      true
    } else {
      false
    }
  )

  let GIT_BG = "#C4A000"
  let GIT_FG = "#000000"
  # let TERM_BG = "#0C0C0C"

  # The multi-color fg colors are good if you just have a black background

  let AHEAD_ICON = (get_icon_by_name AHEAD_ICON)
  # let A_COLOR = (ansi { fg:"#00ffff" bg: ($GIT_BG) })
  let A_COLOR = (ansi { fg: ($GIT_FG) bg: ($GIT_BG) })

  let BEHIND_ICON = (get_icon_by_name BEHIND_ICON)
  # let B_COLOR = (ansi { fg:"#00ffff" bg: ($GIT_BG) })
  let B_COLOR = (ansi { fg: ($GIT_FG) bg: ($GIT_BG) })

  let INDEX_CHANGE_ICON = (get_icon_by_name INDEX_CHANGE_ICON)
  # let I_COLOR = (ansi { fg:"#00ff00" bg: ($GIT_BG) })
  let I_COLOR = (ansi { fg: ($GIT_FG) bg: ($GIT_BG) })

  let CONFLICTED_CHANGE_ICON = (get_icon_by_name CONFLICTED_CHANGE_ICON)
  # let C_COLOR = (ansi { fg:"#ff0000" bg: ($GIT_BG) })
  let C_COLOR = (ansi { fg: ($GIT_FG) bg: ($GIT_BG) })

  let WT_CHANGE_ICON = (get_icon_by_name WT_CHANGE_ICON)
  # let W_COLOR = (ansi { fg:"#ff00ff" bg: ($GIT_BG) })
  let W_COLOR = (ansi { fg: ($GIT_FG) bg: ($GIT_BG) })

  let UNTRACKED_CHANGE_ICON = (get_icon_by_name UNTRACKED_CHANGE_ICON)
  # let U_COLOR = (ansi { fg:"#ffff00" bg: ($GIT_BG) })
  let U_COLOR = (ansi { fg: ($GIT_FG) bg: ($GIT_BG) })

  let NO_CHANGE_ICON = (get_icon_by_name NO_CHANGE_ICON)
  # let N_COLOR = (ansi { fg:"#00ff00" bg: ($GIT_BG) })
  let N_COLOR = (ansi { fg: ($GIT_FG) bg: ($GIT_BG) })

  let HAS_CHANGE_ICON = (get_icon_by_name HAS_CHANGE_ICON)
  # let H_COLOR = (ansi { fg:"#ff0000" bg: ($GIT_BG) attr: b })
  let H_COLOR = (ansi { fg: ($GIT_FG) bg: ($GIT_BG) attr: b })

  let INSERT_SYMBOL_ICON = (get_icon_by_name INSERT_SYMBOL_ICON)
  # let S_COLOR = (ansi { fg:"#00ffff" bg: ($GIT_BG) })
  let S_COLOR = (ansi { fg: ($GIT_FG) bg: ($GIT_BG) })

  let R = (ansi reset)

  let repo_status = (
    $"(
      if ($ahead_cnt > 0) {$'($A_COLOR)($AHEAD_ICON)($ahead_cnt)($R)'}
    )(
      if ($behind_cnt > 0) {$'($B_COLOR)($BEHIND_ICON)($behind_cnt)($R)'}
    )(
      if ($index_change_cnt > 0) {$'($I_COLOR)($INDEX_CHANGE_ICON)($index_change_cnt)($R)'}
    )(
      if ($conflicted_cnt > 0) {$'($C_COLOR)($CONFLICTED_CHANGE_ICON)($conflicted_cnt)($R)'}
    )(
      if ($wt_change_cnt > 0) {$'($W_COLOR)($WT_CHANGE_ICON)($wt_change_cnt)($R)'}
    )(
      if ($untracked_cnt > 0) {$'($U_COLOR)($UNTRACKED_CHANGE_ICON)($untracked_cnt)($R)'}
    )(
      if $has_no_changes {$'($N_COLOR)($NO_CHANGE_ICON)($R)'} else {$'($H_COLOR)($HAS_CHANGE_ICON)($R)'}
    )"
  )

  $repo_status
}

def git_prompt_info [gs] {
  let branch_name = (get_branch_name $gs)

  if ($branch_name == "") {
    ""
  } else {
    let index_change_cnt = (get_index_change_count $gs)
    let wt_change_cnt = (get_working_tree_count $gs)
    let conflicted_cnt = (get_conflicted_count $gs)
    let untracked_cnt = (get_untracked_count $gs)
    let is_dirty = (
      ($index_change_cnt > 0) or
      ($wt_change_cnt > 0) or
      ($conflicted_cnt > 0) or
      ($untracked_cnt > 0)
    )

    let R = (ansi reset)
    let blue_bold = (ansi { fg: "blue" attr: b })
    let red_bold = (ansi { fg: "red" attr: b })

    $" ($blue_bold)(($red_bold)($branch_name)(if $is_dirty { '*' } else { '' })($blue_bold))($R)"
  }
}

def git_left_prompt [gs os] {
  let is_root = (is-admin)
  let R = (ansi reset)
  let user_color = (if $is_root { "red" } else { "green" })
  let user_fmt = (ansi { fg: $user_color attr: b })
  let path_fmt = (ansi { fg: "blue" attr: b })
  let prompt_char = (if $is_root { "#" } else { "$" })

  let cwd_piece = (
    if ($env.PWD == $nu.home-dir) {
      "~"
    } else {
      $env.PWD | path basename
    }
  )

  let git_info = (git_prompt_info $gs)
  $"($user_fmt)(whoami)($R):($path_fmt)($cwd_piece)/($R)($git_info) ($prompt_char)"
}

def git_right_prompt [gs os] {
  $"[(date now | format date '%H:%M:%S')]"
}

export def git_prompt [] {
  let gs = (gstat)
  let os = $nu.os-info
  let left_prompt = (git_left_prompt $gs $os)
  let right_prompt = (git_right_prompt $gs $os)
  let use_ansi = (config use-colors)

  # set the title of the window/tab
  # Wezterm accepts:
  # osc0 \x1b]0;
  # osc1 \x1b]1;
  # osc2 \x1b]2;
  # the typical way to set the terminal title is:
  # osc2 some_string bel aka (char osc)2;($some_string)(char bel) or "\u001b]2;($some_string)\a"
  # bel is escape \a or \x7 or \u0007
  # but i've also seen it as
  # osc2 some_string string_terminator aka (char osc)2;($some_string)(ansi st) or "\u001b];($some_string)\\"
  # where string_terminator is \
  # so you might want to play around with these settings a bit

  #let abbrev = ((path_abbrev_if_needed (home_abbrev $os.name) 30) | ansi strip)

  # $"\u001b]0;($abbrev)"
  # note that this isn't ending properly with a bel or a st, that's
  # because it makes the string echo to the screen as an empty line

  # turning off now since a similar thing is built into nushell + it breaks kitty
  #$"(ansi osc)2;($abbrev)"

  # return in record literal syntax to be used kind of like a tuple
  # so we don't have to run this script more than once per prompt
  {
    left_prompt: (if $use_ansi { $left_prompt } else { $left_prompt | ansi strip })
    right_prompt: (if $use_ansi { $right_prompt } else { $right_prompt | ansi strip })
  }
  #
  # in the config.nu you would do something like
  # use "c:\some\path\to\nu_scripts\prompt\oh-my.nu" git_prompt
  # $env.PROMPT_COMMAND = { (git_prompt).left_prompt }
  # $env.PROMPT_COMMAND_RIGHT = { (git_prompt).right_prompt }
  # $env.PROMPT_INDICATOR = " "
}