#!/usr/bin/env bash
set -euo pipefail

# ── Colors ──────────────────────────────────────────────────────────────────
BOLD='\033[1m'
DIM='\033[2m'
CYAN='\033[36m'
GREEN='\033[32m'
YELLOW='\033[33m'
RED='\033[31m'
BLUE='\033[34m'
MAGENTA='\033[35m'
RESET='\033[0m'
ARROW="${CYAN}>${RESET}"

# Readline-safe colors (for read -p prompts)
RL_BOLD=$'\001\033[1m\002'
RL_DIM=$'\001\033[2m\002'
RL_RED=$'\001\033[31m\002'
RL_RESET=$'\001\033[0m\002'

REMOTE_HOST="git-PM7"
REMOTE_PATH="repos"

# ── Auto .gitignore ───────────────────────────────────────────────────────
generate_gitignore() {
    local stacks=()
    local gi=""

    # ── Common ──
    gi+="# ── OS / Éditeurs ──\n"
    gi+=".DS_Store\nThumbs.db\n"
    gi+=".vscode/\n.idea/\n*.swp\n*.swo\n*~\n"
    gi+="\n# ── Dotfiles sensibles ──\n"
    gi+=".env\n.env.*\n.secret\n.secrets/\n"
    gi+="\n# ── Logs / Temp ──\n"
    gi+="*.log\n*.tmp\n*.bak\n*.pid\n"

    # ── Detect Svelte / Node ──
    if [[ -f "package.json" ]]; then
        stacks+=("Node/Svelte")
        gi+="\n# ── Node / Svelte ──\n"
        gi+="node_modules/\nbuild/\ndist/\n.svelte-kit/\n"
        gi+=".vercel/\n.netlify/\n"
        gi+=".pnpm-store/\n"
        # Capacitor
        if [[ -f "capacitor.config.ts" || -f "capacitor.config.json" ]]; then
            stacks+=("Capacitor")
            gi+="\n# ── Capacitor ──\n"
            gi+="android/\nios/\n"
        fi
    fi

    # ── Detect Rust ──
    if [[ -f "Cargo.toml" ]] || ls */Cargo.toml &>/dev/null; then
        stacks+=("Rust")
        gi+="\n# ── Rust ──\n"
        gi+="target/\n*.rs.bk\n"
    fi

    # ── Detect Go ──
    if [[ -f "go.mod" ]] || ls */go.mod &>/dev/null; then
        stacks+=("Go")
        gi+="\n# ── Go ──\n"
        gi+="bin/\nvendor/\n"
    fi

    # ── Detect Flutter ──
    if [[ -f "pubspec.yaml" ]] || ls */pubspec.yaml &>/dev/null; then
        stacks+=("Flutter")
        gi+="\n# ── Flutter / Dart ──\n"
        gi+=".dart_tool/\n.flutter-plugins\n.flutter-plugins-dependencies\n"
        gi+=".packages\nbuild/\n*.iml\n"
    fi

    # ── Write ──
    echo -e "$gi" > .gitignore

    if [[ ${#stacks[@]} -gt 0 ]]; then
        local IFS=', '
        echo -e "  ${DIM}Stack détectée : ${MAGENTA}${stacks[*]}${RESET}"
    else
        echo -e "  ${DIM}Aucune stack détectée, .gitignore minimal créé.${RESET}"
    fi
    echo -e "  ${GREEN}.gitignore généré.${RESET}"
}

# ── Helper: check/clean remote bare repo ─────────────────────────────────
# Usage: ensure_bare_repo "repo_name"
# Returns 0 to continue, 1 to abort
ensure_bare_repo() {
    local repo_name="$1"
    local remote_path="${REMOTE_PATH}/${repo_name}.git"

    if ssh "$REMOTE_HOST" "test -d ${remote_path}" 2>/dev/null; then
        echo ""
        echo -e "  ${YELLOW}Le dépôt bare '${repo_name}.git' existe déjà sur ${REMOTE_HOST}.${RESET}"
        echo ""
        read -erp "  ${RL_RED}${RL_BOLD}Supprimer et recréer ? (oui/non) :${RL_RESET} " confirm
        if [[ "$confirm" != "oui" ]]; then
            echo -e "  ${DIM}Annulé.${RESET}"
            return 1
        fi
        echo -e "  ${DIM}Suppression de ${remote_path} sur ${REMOTE_HOST}...${RESET}"
        ssh "$REMOTE_HOST" "rm -rf ${remote_path}"
    fi

    echo -e "  ${DIM}Création du dépôt bare sur ${REMOTE_HOST}...${RESET}"
    ssh "$REMOTE_HOST" "git init --bare ${remote_path}"
    return 0
}

# ── Init if not a git repo ──────────────────────────────────────────────────
if ! git rev-parse --show-toplevel &>/dev/null; then
    default_name=$(basename "$PWD")
    echo ""
    echo -e "  ${BOLD}Initialiser un nouveau dépôt ?${RESET}"
    echo ""
    # Inline menu (can't use menu_select with nameref before init)
    options=("Oui, initialiser" "Quitter")
    selected=0
    count=${#options[@]}
    tput civis 2>/dev/null || true
    for i in "${!options[@]}"; do
        if (( i == selected )); then
            echo -e "  ${ARROW} ${BOLD}${options[$i]}${RESET}"
        else
            echo -e "    ${DIM}${options[$i]}${RESET}"
        fi
    done
    while true; do
        IFS= read -rsn1 key
        if [[ "$key" == $'\x1b' ]]; then read -rsn2 -t 0.1 key2 || true; key+="$key2"; fi
        case "$key" in
            $'\x1b[A') (( selected > 0 )) && selected=$((selected - 1)) ;;
            $'\x1b[B') (( selected < count - 1 )) && selected=$((selected + 1)) ;;
            '') break ;;
            q|Q) tput cnorm 2>/dev/null || true; echo -e "\n  ${DIM}Bye !${RESET}"; exit 0 ;;
        esac
        echo -en "\033[${count}A"
        for i in "${!options[@]}"; do
            echo -en "\033[2K"
            if (( i == selected )); then
                echo -e "  ${ARROW} ${BOLD}${options[$i]}${RESET}"
            else
                echo -e "    ${DIM}${options[$i]}${RESET}"
            fi
        done
    done
    tput cnorm 2>/dev/null || true

    if (( selected != 0 )); then
        echo -e "\n  ${DIM}Bye !${RESET}"
        exit 0
    fi

    echo ""
    read -erp "  ${RL_BOLD}Nom du dépôt${RL_RESET} ${RL_DIM}(${default_name})${RL_RESET}${RL_BOLD} :${RL_RESET} " repo_name
    repo_name="${repo_name:-$default_name}"

    echo ""
    echo -e "  ${DIM}git init...${RESET}"
    git init -b main .
    if ! ensure_bare_repo "$repo_name"; then
        rm -rf .git
        exit 0
    fi
    git remote add origin "${REMOTE_HOST}:${REMOTE_PATH}/${repo_name}.git"
    generate_gitignore
    echo -e "  ${DIM}Commit initial...${RESET}"
    git add -A
    git commit -m "Initial commit"
    echo -e "  ${DIM}Push main → origin...${RESET}"
    git push -u origin main
    echo ""
    echo -e "  ${GREEN}Dépôt initialisé et poussé.${RESET}"
    echo -e "  ${DIM}Local  : ${PWD}${RESET}"
    echo -e "  ${DIM}Remote : ${REMOTE_HOST}:${REMOTE_PATH}/${repo_name}.git${RESET}"
    echo ""
    echo -e "  ${DIM}Appuie sur une touche...${RESET}"
    read -rsn1
fi

cd "$(git rev-parse --show-toplevel 2>/dev/null)"

# ── Initial fetch ────────────────────────────────────────────────────────────
git fetch --prune -q origin 2>/dev/null || true

# ── Helper: print git info banner ───────────────────────────────────────────
print_header() {
    local branch
    branch=$(git branch --show-current 2>/dev/null || echo "HEAD détachée")
    local local_branches remote_branches all_branches
    local_branches=$(git branch --format='%(refname:short)' | sort)
    remote_branches=$(git branch -r --format='%(refname:short)' 2>/dev/null \
        | grep '^origin/' | grep -v '/HEAD$' | sed 's|^origin/||' | sort)
    all_branches=$(printf '%s\n%s' "$local_branches" "$remote_branches" \
        | grep -v '^$' | sort -u)
    local status
    status=$(git status --short 2>/dev/null)
    local ahead_behind
    ahead_behind=$(git rev-list --left-right --count HEAD...@{upstream} 2>/dev/null || echo "")
    local last_commit
    last_commit=$(git log --oneline -1 2>/dev/null || echo "(aucun commit)")

    echo ""
    echo -e "  ${BOLD}Git — $(basename "$PWD")${RESET}    ${BLUE}main${RESET}${DIM} : local${RESET}  ${GREEN}origin${RESET}${DIM} : remote${RESET}"
    echo -e "  ${DIM}$(printf '%.0s─' {1..50})${RESET}"
    # Branch table: column width
    local max_bname=5
    while IFS= read -r b; do
        [[ -z "$b" ]] && continue
        (( ${#b} > max_bname )) && max_bname=${#b}
    done <<< "$all_branches"
    local col_w=$((max_bname + 4))
    local tbl_prefix="               "

    # Column headers
    echo -ne "  ${BOLD}Branches :${RESET}   "
    echo -ne "${BOLD}$(printf '%-*s' "$col_w" "Local")${RESET}"
    echo -e  "${BOLD}Remote${RESET}"
    echo -ne "$tbl_prefix"
    echo -ne "${DIM}$(printf '%-*s' "$col_w" "─────")${RESET}"
    echo -e  "${DIM}──────${RESET}"

    # Branch rows
    local in_local in_remote
    while IFS= read -r b; do
        [[ -z "$b" ]] && continue
        in_local=false
        in_remote=false
        grep -qx "$b" <<< "$local_branches" && in_local=true
        grep -qx "$b" <<< "$remote_branches" && in_remote=true

        echo -ne "$tbl_prefix"

        # Local column
        if $in_local; then
            if [[ "$b" == "$branch" ]]; then
                echo -ne "${RED}* $(printf '%-*s' "$((col_w - 2))" "$b")${RESET}"
            else
                echo -ne "  ${DIM}$(printf '%-*s' "$((col_w - 2))" "$b")${RESET}"
            fi
        else
            printf '%*s' "$col_w" ""
        fi

        # Remote column
        if $in_remote; then
            if ! $in_local; then
                echo -e "${CYAN}○ ${b}${RESET}"
            else
                echo -e "  ${DIM}${b}${RESET}"
            fi
        else
            echo ""
        fi
    done <<< "$all_branches"

    if [[ -n "$ahead_behind" ]]; then
        local ahead behind
        ahead=$(echo "$ahead_behind" | awk '{print $1}')
        behind=$(echo "$ahead_behind" | awk '{print $2}')
        local sync_label=""
        (( ahead > 0 ))  && sync_label+="${YELLOW}+${ahead} ahead${RESET} "
        (( behind > 0 )) && sync_label+="${RED}-${behind} behind${RESET} "
        [[ -z "$sync_label" ]] && sync_label="${GREEN}synced${RESET}"
        echo -e "  ${BOLD}Remote :${RESET}   ${sync_label}"
    fi

    echo -e "  ${BOLD}Commit :${RESET}   ${DIM}${last_commit}${RESET}"

    if [[ -n "$status" ]]; then
        local modified added untracked
        modified=$(echo "$status" | grep -c '^ M\|^MM\|^ D' || true)
        added=$(echo "$status" | grep -c '^A \|^M ' || true)
        untracked=$(echo "$status" | grep -c '^??' || true)
        echo -ne "  ${BOLD}Fichiers :${RESET} "
        local parts=()
        (( modified > 0 ))  && parts+=("${YELLOW}${modified} modifié(s)${RESET}")
        (( added > 0 ))     && parts+=("${GREEN}${added} stagé(s)${RESET}")
        (( untracked > 0 )) && parts+=("${DIM}${untracked} non suivi(s)${RESET}")
        local IFS=', '
        echo -e "${parts[*]}"
    else
        echo -e "  ${BOLD}Fichiers :${RESET} ${GREEN}copie propre${RESET}"
    fi
    echo -e "  ${DIM}$(printf '%.0s─' {1..50})${RESET}"
}

# ── Helper: arrow-key menu ──────────────────────────────────────────────────
# Usage: menu_select result_var "Option1" "---" "Option2" ...
# Items named "---" are rendered as separators and skipped during navigation.
menu_select() {
    local -n _result=$1
    shift
    local options=("$@")
    local count=${#options[@]}
    local sep_line="    ${DIM}$(printf '%.0s─' {1..30})${RESET}"

    # Find first selectable item
    local selected=0
    while (( selected < count )) && [[ "${options[$selected]}" == "---" ]]; do
        selected=$((selected + 1))
    done

    # Helper: move to next selectable item in a direction
    _next_selectable() {
        local pos=$1 dir=$2
        while true; do
            pos=$((pos + dir))
            (( pos < 0 || pos >= count )) && return 1
            [[ "${options[$pos]}" != "---" ]] && { echo "$pos"; return 0; }
        done
    }

    # Hide cursor
    tput civis 2>/dev/null || true

    # Print menu
    for i in "${!options[@]}"; do
        if [[ "${options[$i]}" == "---" ]]; then
            echo -e "$sep_line"
        elif (( i == selected )); then
            echo -e "  ${ARROW} ${BOLD}${options[$i]}${RESET}"
        else
            echo -e "    ${DIM}${options[$i]}${RESET}"
        fi
    done

    while true; do
        IFS= read -rsn1 key
        if [[ "$key" == $'\x1b' ]]; then
            read -rsn2 -t 0.1 key2 || true
            key+="$key2"
        fi

        case "$key" in
            $'\x1b[A') # Up
                local next
                next=$(_next_selectable "$selected" -1) && selected=$next
                ;;
            $'\x1b[B') # Down
                local next
                next=$(_next_selectable "$selected" 1) && selected=$next
                ;;
            '') # Enter
                break
                ;;
            q|Q)
                tput cnorm 2>/dev/null || true
                _result=-1
                return
                ;;
        esac

        # Redraw
        echo -en "\033[${count}A"
        for i in "${!options[@]}"; do
            echo -en "\033[2K"
            if [[ "${options[$i]}" == "---" ]]; then
                echo -e "$sep_line"
            elif (( i == selected )); then
                echo -e "  ${ARROW} ${BOLD}${options[$i]}${RESET}"
            else
                echo -e "    ${DIM}${options[$i]}${RESET}"
            fi
        done
    done

    tput cnorm 2>/dev/null || true
    _result=$selected
}

# ── Actions ─────────────────────────────────────────────────────────────────

action_new_branch() {
    echo ""
    read -erp "  ${RL_BOLD}Nom de la nouvelle branche :${RL_RESET} " name
    [[ -z "$name" ]] && { echo -e "  ${RED}Annulé.${RESET}"; return; }
    git checkout -b "$name"
    echo -e "  ${GREEN}Branche '${name}' créée et activée.${RESET}"
}

action_commit() {
    local status
    status=$(git status --short 2>/dev/null)
    if [[ -z "$status" ]]; then
        echo -e "\n  ${YELLOW}Rien à commiter.${RESET}"
        return
    fi

    echo ""
    echo -e "  ${BOLD}Modifications :${RESET}"
    git status --short | sed 's/^/    /'
    echo ""

    echo -e "  ${BOLD}Ajouter tous les fichiers modifiés ?${RESET}"
    local choice
    menu_select choice "Oui (git add -A)" "Non, choisir manuellement" "Annuler"
    case $choice in
        0)  git add -A ;;
        1)
            echo ""
            read -erp "  ${RL_BOLD}Fichiers à ajouter (séparés par espace) :${RL_RESET} " files
            [[ -z "$files" ]] && { echo -e "  ${RED}Annulé.${RESET}"; return; }
            eval git add $files
            ;;
        *)  echo -e "  ${RED}Annulé.${RESET}"; return ;;
    esac

    echo ""
    read -erp "  ${RL_BOLD}Message du commit :${RL_RESET} " msg
    [[ -z "$msg" ]] && { echo -e "  ${RED}Annulé.${RESET}"; return; }

    git commit -m "$msg"
    echo -e "\n  ${GREEN}Commit créé.${RESET}"
}

action_merge() {
    local branch
    branch=$(git branch --show-current)

    if [[ "$branch" == "main" ]]; then
        echo -e "\n  ${YELLOW}Tu es déjà sur main. Choisis une branche à merger :${RESET}"
        local branches=()
        while IFS= read -r b; do
            [[ "$b" != "main" ]] && branches+=("$b")
        done < <(git branch --format='%(refname:short)')

        if [[ ${#branches[@]} -eq 0 ]]; then
            echo -e "  ${DIM}Aucune autre branche disponible.${RESET}"
            return
        fi

        branches+=("Annuler")
        local choice
        menu_select choice "${branches[@]}"
        (( choice < 0 || choice >= ${#branches[@]} - 1 )) && return
        local target="${branches[$choice]}"
        git merge "$target"
        echo -e "\n  ${GREEN}'${target}' mergée dans main.${RESET}"
    else
        echo ""
        echo -e "  ${BOLD}Merger '${branch}' dans main ?${RESET}"
        local choice
        menu_select choice "Oui" "Annuler"
        [[ $choice -ne 0 ]] && { echo -e "  ${RED}Annulé.${RESET}"; return; }

        git checkout main
        git merge "$branch"
        echo -e "\n  ${GREEN}'${branch}' mergée dans main.${RESET}"
    fi
}

action_push() {
    local branch
    branch=$(git branch --show-current)

    if [[ "$branch" != "main" ]]; then
        echo -e "\n  ${YELLOW}Tu n'es pas sur main (branche: ${branch}).${RESET}"
        echo -e "  ${BOLD}Basculer sur main et pousser ?${RESET}"
        local choice
        menu_select choice "Oui" "Annuler"
        [[ $choice -ne 0 ]] && { echo -e "  ${RED}Annulé.${RESET}"; return; }
        git checkout main
    fi

    echo -e "\n  ${BOLD}Pousser main sur origin...${RESET}"
    git push origin main
    echo -e "  ${GREEN}Poussé.${RESET}"

    # Nettoyage des branches locales déjà mergées dans main
    local merged
    merged=$(git branch --merged main --format='%(refname:short)' | grep -v '^main$' || true)
    if [[ -n "$merged" ]]; then
        echo ""
        echo -e "  ${BOLD}Branches mergées à supprimer :${RESET}"
        while IFS= read -r b; do
            echo -e "    ${DIM}${b}${RESET}"
        done <<< "$merged"
        while IFS= read -r b; do
            git branch -d "$b" &>/dev/null
            git push origin --delete "$b" &>/dev/null || true
        done <<< "$merged"
        echo -e "  ${GREEN}Nettoyé (local + remote).${RESET}"
    fi
}

action_backup() {
    local branch
    branch=$(git branch --show-current)
    if [[ "$branch" == "main" ]]; then
        echo -e "\n  ${YELLOW}Tu es sur main, utilise 'Pousser main → origin'.${RESET}"
        return
    fi
    echo -e "\n  ${BOLD}Backup ${branch} sur origin...${RESET}"
    git push origin "$branch"
    echo -e "  ${GREEN}${branch} poussée sur origin.${RESET}"
}

action_switch() {
    local branch
    branch=$(git branch --show-current)
    local branches=()
    while IFS= read -r b; do
        [[ "$b" != "$branch" ]] && branches+=("$b")
    done < <(git branch --format='%(refname:short)')

    if [[ ${#branches[@]} -eq 0 ]]; then
        echo -e "\n  ${DIM}Aucune autre branche.${RESET}"
        return
    fi

    branches+=("Annuler")
    echo -e "\n  ${BOLD}Changer de branche :${RESET}"
    local choice
    menu_select choice "${branches[@]}"
    (( choice < 0 || choice >= ${#branches[@]} - 1 )) && return
    git checkout "${branches[$choice]}"
    echo -e "\n  ${GREEN}Basculé sur '${branches[$choice]}'.${RESET}"
}

action_fetch() {
    echo -e "\n  ${BOLD}Récupération des branches distantes...${RESET}"
    git fetch --prune origin
    echo -e "  ${GREEN}Fetch terminé.${RESET}"
}

action_reinit() {
    echo ""
    echo -e "  ${RED}${BOLD}⚠ Réinitialisation complète du dépôt git${RESET}"
    echo -e "  ${DIM}Cela va supprimer tout l'historique git local${RESET}"
    echo -e "  ${DIM}et recréer un dépôt vierge avec un nouveau remote.${RESET}"
    echo ""

    if git remote get-url origin &>/dev/null; then
        echo -e "  ${DIM}Remote actuel : $(git remote get-url origin)${RESET}"
        echo ""
    fi

    read -erp "  ${RL_RED}${RL_BOLD}Confirmer en tapant 'oui' :${RL_RESET} " confirm
    if [[ "$confirm" != "oui" ]]; then
        echo -e "  ${DIM}Annulé.${RESET}"
        return
    fi

    local default_name
    default_name=$(basename "$PWD")
    echo ""
    read -erp "  ${RL_BOLD}Nom du dépôt${RL_RESET} ${RL_DIM}(${default_name})${RL_RESET}${RL_BOLD} :${RL_RESET} " repo_name
    repo_name="${repo_name:-$default_name}"

    if ! ensure_bare_repo "$repo_name"; then
        return
    fi

    echo ""
    echo -e "  ${DIM}Suppression de .git...${RESET}"
    rm -rf .git
    echo -e "  ${DIM}git init...${RESET}"
    git init -b main .
    git remote add origin "${REMOTE_HOST}:${REMOTE_PATH}/${repo_name}.git"
    generate_gitignore
    echo -e "  ${DIM}Commit initial...${RESET}"
    git add -A
    git commit -m "Initial commit"
    echo -e "  ${DIM}Push main → origin...${RESET}"
    git push -u origin main
    echo ""
    echo -e "  ${GREEN}Dépôt réinitialisé et poussé.${RESET}"
    echo -e "  ${DIM}Local  : ${PWD}${RESET}"
    echo -e "  ${DIM}Remote : ${REMOTE_HOST}:${REMOTE_PATH}/${repo_name}.git${RESET}"
}

action_history() {
    local repo_name
    repo_name=$(basename "$(git rev-parse --show-toplevel)")
    local today
    today=$(date +%Y-%m-%d)
    local filepath="docs/git-${repo_name}-${today}.md"

    mkdir -p docs

    local branch
    branch=$(git branch --show-current 2>/dev/null || echo "?")
    local remote_url
    remote_url=$(git remote get-url origin 2>/dev/null || echo "aucun")

    {
        echo "# Git — ${repo_name}"
        echo ""
        echo "- **Date** : ${today}"
        echo "- **Branche** : ${branch}"
        echo "- **Remote** : ${remote_url}"
        echo ""
        echo "## Historique (10 derniers jours)"
        echo ""

        local log
        log=$(git log --all --since="10 days ago" \
            --format="### %h — %s%n- **Date** : %ci%n- **Auteur** : %an%n- **Branche** : %D%n" \
            2>/dev/null)

        if [[ -z "$log" ]]; then
            echo "_Aucune activité sur les 10 derniers jours._"
        else
            echo "$log"
        fi

        echo "---"
        echo ""
        echo "## Branches"
        echo ""
        git branch -a --format='- `%(refname:short)`' 2>/dev/null
        echo ""

        echo "## Statistiques"
        echo ""
        local total
        total=$(git rev-list --all --count 2>/dev/null || echo "0")
        local contributors
        contributors=$(git log --all --format='%an' 2>/dev/null | sort -u | sed 's/^/- /')
        echo "- **Commits total** : ${total}"
        echo "- **Contributeurs** :"
        echo "$contributors"
    } > "$filepath"

    echo ""
    echo -e "  ${GREEN}Fichier généré : ${filepath}${RESET}"
}

# ── Main loop ───────────────────────────────────────────────────────────────

while true; do
    clear
    print_header
    echo ""
    echo -e "  ${BOLD}Actions :${RESET}  ${DIM}(flèches + entrée, q = quitter)${RESET}"
    echo ""

    current_branch=$(git branch --show-current 2>/dev/null || echo "?")
    cb="${RED}${current_branch}${RESET}"
    mn="${BLUE}main${RESET}"
    or="${GREEN}origin${RESET}"

    if [[ "$current_branch" == "main" ]]; then
        merge_label="Merger dans ${mn}"
    else
        merge_label="Merger ${cb} dans ${mn}"
    fi

    choice=0
    if [[ "$current_branch" != "main" ]]; then
        menu_select choice \
            "Nouvelle branche" \
            "Commiter ${cb}" \
            "Changer de branche" \
            "Rafraîchir (fetch)" \
            "$merge_label" \
            "Backup ${cb} → ${or}" \
            "Pousser ${mn} → ${or}" \
            "Historique" \
            "---" \
            "Initialisation Git" \
            "---" \
            "Quitter"

        case $choice in
            0) action_new_branch ;;
            1) action_commit ;;
            2) action_switch ;;
            3) action_fetch ;;
            4) action_merge ;;
            5) action_backup ;;
            6) action_push ;;
            7) action_history ;;
            9) action_reinit ;;
            *) echo -e "\n  ${DIM}Bye !${RESET}"; exit 0 ;;
        esac
    else
        menu_select choice \
            "Nouvelle branche" \
            "Commiter ${cb}" \
            "Changer de branche" \
            "Rafraîchir (fetch)" \
            "$merge_label" \
            "Pousser ${mn} → ${or}" \
            "Historique" \
            "---" \
            "Initialisation Git" \
            "---" \
            "Quitter"

        case $choice in
            0) action_new_branch ;;
            1) action_commit ;;
            2) action_switch ;;
            3) action_fetch ;;
            4) action_merge ;;
            5) action_push ;;
            6) action_history ;;
            8) action_reinit ;;
            *) echo -e "\n  ${DIM}Bye !${RESET}"; exit 0 ;;
        esac
    fi

    echo ""
    echo -e "  ${DIM}Appuie sur une touche...${RESET}"
    read -rsn1
done
