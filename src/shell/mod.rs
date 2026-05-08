/// Shell 类型
#[derive(Debug, Clone, Copy)]
pub enum ShellType {
    Bash,
    Zsh,
    Fish,
    PowerShell,
}

impl ShellType {
    pub fn detect() -> Self {
        if cfg!(target_os = "windows") {
            return ShellType::PowerShell;
        }
        let shell = std::env::var("SHELL").unwrap_or_default();
        if shell.contains("fish") {
            ShellType::Fish
        } else if shell.contains("zsh") {
            ShellType::Zsh
        } else {
            ShellType::Bash
        }
    }

    #[allow(dead_code)]
    pub fn name(&self) -> &'static str {
        match self {
            ShellType::Bash => "bash",
            ShellType::Zsh => "zsh",
            ShellType::Fish => "fish",
            ShellType::PowerShell => "powershell",
        }
    }
}

/// 生成 Shell 初始化脚本
pub fn generate_init_script(shell: ShellType) -> String {
    match shell {
        ShellType::Bash | ShellType::Zsh => BASH_INIT_SCRIPT.to_string(),
        ShellType::Fish => FISH_INIT_SCRIPT.to_string(),
        ShellType::PowerShell => POWERSHELL_INIT_SCRIPT.to_string(),
    }
}

/// 密钥名称转环境变量名
pub fn key_to_env_var(name: &str) -> String {
    name.to_uppercase()
        .replace('-', "_")
        .replace(' ', "_")
        .replace('.', "_")
}

/// 生成环境变量导出命令
pub fn generate_export_command(name: &str, value: &str, shell: ShellType) -> String {
    let env_name = key_to_env_var(name);
    match shell {
        ShellType::Bash | ShellType::Zsh => format!("export {}=\"{}\"", env_name, value),
        ShellType::Fish => format!("set -gx {} \"{}\"", env_name, value),
        ShellType::PowerShell => format!("$env:{} = \"{}\"", env_name, value),
    }
}

const BASH_INIT_SCRIPT: &str = r#"# apikey_etorer shell integration
# Add to ~/.bashrc or ~/.zshrc:
#   eval "$(apikey-etorer shell init)"

apikey_etorer_get() {
    local key_name="$1"
    local value
    value=$(apikey-etorer key get "$key_name" --copy 2>/dev/null)
    if [ $? -eq 0 ]; then
        echo "$value"
    else
        echo "Error: Failed to get key '$key_name'" >&2
        return 1
    fi
}

apikey_etorer_set_env() {
    local key_name="$1"
    local env_name="${2:-$(echo "$key_name" | tr '[:lower:]-' '[:upper:]_')}"
    local value
    value=$(apikey-etorer key get "$key_name" 2>/dev/null)
    if [ $? -eq 0 ]; then
        export "$env_name=$value"
    else
        echo "Error: Failed to get key '$key_name'" >&2
        return 1
    fi
}

alias ake='apikey-etorer'
alias akget='apikey_etorer_get'
alias akenv='apikey_etorer_set_env'
"#;

const FISH_INIT_SCRIPT: &str = r#"# apikey_etorer Fish shell integration
# Add to ~/.config/fish/config.fish:
#   apikey-etorer shell init | source

function apikey_etorer_get
    set key_name $argv[1]
    set value (apikey-etorer key get $key_name --copy 2>/dev/null)
    if test $status -eq 0
        echo $value
    else
        echo "Error: Failed to get key '$key_name'" >&2
        return 1
    end
end

function apikey_etorer_set_env
    set key_name $argv[1]
    set env_name $argv[2]
    if test -z "$env_name"
        set env_name (echo $key_name | tr '[:lower:]-' '[:upper:]_')
    end
    set value (apikey-etorer key get $key_name 2>/dev/null)
    if test $status -eq 0
        set -gx $env_name $value
    else
        echo "Error: Failed to get key '$key_name'" >&2
        return 1
    end
end

abbr -a ake apikey-etorer
abbr -a akget apikey_etorer_get
abbr -a akenv apikey_etorer_set_env
"#;

const POWERSHELL_INIT_SCRIPT: &str = r#"# apikey_etorer PowerShell integration
# Add to $PROFILE:
#   apikey-etorer shell init | Invoke-Expression

function Get-ApiKeyValue {
    param([string]$KeyName)
    $value = apikey-etorer key get $KeyName 2>$null
    if ($LASTEXITCODE -eq 0) {
        return $value
    } else {
        Write-Error "Failed to get key '$KeyName'"
    }
}

function Set-ApiKeyEnv {
    param([string]$KeyName, [string]$EnvName)
    if (-not $EnvName) {
        $EnvName = $KeyName.ToUpper() -replace '-', '_' -replace ' ', '_'
    }
    $value = Get-ApiKeyValue $KeyName
    if ($value) {
        [Environment]::SetEnvironmentVariable($EnvName, $value, "Process")
    }
}

Set-Alias -Name ake -Value apikey-etorer
Set-Alias -Name akget -Value Get-ApiKeyValue
Set-Alias -Name akenv -Value Set-ApiKeyEnv
"#;