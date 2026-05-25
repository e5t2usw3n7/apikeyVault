import { useQuery } from "@tanstack/react-query";
import { listKeys, listGroups } from "@/api";
import { KeyRound, FolderOpen, Layers, TrendingUp } from "lucide-react";

export function DashboardPage() {
  const { data: keys = [] } = useQuery({ queryKey: ["keys"], queryFn: listKeys });
  const { data: groups = [] } = useQuery({ queryKey: ["groups"], queryFn: listGroups });

  const envStats = keys.reduce((acc, key) => {
    acc[key.environment] = (acc[key.environment] || 0) + 1;
    return acc;
  }, {} as Record<string, number>);

  const providerStats = keys.reduce((acc, key) => {
    acc[key.provider] = (acc[key.provider] || 0) + 1;
    return acc;
  }, {} as Record<string, number>);

  const recentKeys = [...keys]
    .sort((a, b) => new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime())
    .slice(0, 5);

  const stats = [
    { label: "总密钥数", value: keys.length, icon: KeyRound, color: "var(--brand-primary)", bg: "rgba(124,58,237,0.1)" },
    { label: "分组数", value: groups.length, icon: FolderOpen, color: "var(--status-success)", bg: "var(--status-success-bg)" },
    { label: "环境数", value: Object.keys(envStats).length, icon: Layers, color: "var(--brand-secondary)", bg: "rgba(6,182,212,0.1)" },
  ];

  return (
    <div className="space-y-6 animate-fade-in">
      <h2 className="text-xl font-bold" style={{ color: "var(--text-primary)" }}>
        仪表盘
      </h2>

      {/* Stat cards */}
      <div className="grid grid-cols-3 gap-4 stagger-children">
        {stats.map((stat) => {
          const Icon = stat.icon;
          return (
            <div
              key={stat.label}
              className="p-5 rounded-xl transition-smooth"
              style={{
                background: "var(--bg-surface)",
                border: "1px solid var(--border-default)",
              }}
            >
              <div className="flex items-center gap-3">
                <div
                  className="w-10 h-10 rounded-lg flex items-center justify-center"
                  style={{ background: stat.bg }}
                >
                  <Icon size={20} style={{ color: stat.color }} />
                </div>
                <div>
                  <div className="text-2xl font-bold" style={{ color: "var(--text-primary)" }}>
                    {stat.value}
                  </div>
                  <div className="text-xs" style={{ color: "var(--text-muted)" }}>
                    {stat.label}
                  </div>
                </div>
              </div>
            </div>
          );
        })}
      </div>

      <div className="grid grid-cols-2 gap-6">
        {/* Environment distribution */}
        <div
          className="p-5 rounded-xl"
          style={{ background: "var(--bg-surface)", border: "1px solid var(--border-default)" }}
        >
          <h3 className="text-sm font-semibold mb-4" style={{ color: "var(--text-primary)" }}>
            环境分布
          </h3>
          <div className="space-y-3">
            {Object.entries(envStats).map(([env, count]) => (
              <div key={env} className="flex items-center gap-3">
                <span className="text-xs w-24" style={{ color: "var(--text-secondary)" }}>
                  {env}
                </span>
                <div className="flex-1 h-2 rounded-full overflow-hidden" style={{ background: "var(--bg-surface-hover)" }}>
                  <div
                    className="h-full rounded-full transition-all duration-500"
                    style={{
                      width: `${(count / keys.length) * 100}%`,
                      background: "var(--brand-primary)",
                    }}
                  />
                </div>
                <span className="text-xs font-medium w-8 text-right" style={{ color: "var(--text-primary)" }}>
                  {count}
                </span>
              </div>
            ))}
          </div>
        </div>

        {/* Provider distribution */}
        <div
          className="p-5 rounded-xl"
          style={{ background: "var(--bg-surface)", border: "1px solid var(--border-default)" }}
        >
          <h3 className="text-sm font-semibold mb-4" style={{ color: "var(--text-primary)" }}>
            提供商分布
          </h3>
          <div className="space-y-3">
            {Object.entries(providerStats)
              .sort(([, a], [, b]) => b - a)
              .slice(0, 5)
              .map(([provider, count]) => (
                <div key={provider} className="flex items-center gap-3">
                  <span className="text-xs w-24 truncate" style={{ color: "var(--text-secondary)" }}>
                    {provider}
                  </span>
                  <div className="flex-1 h-2 rounded-full overflow-hidden" style={{ background: "var(--bg-surface-hover)" }}>
                    <div
                      className="h-full rounded-full transition-all duration-500"
                      style={{
                        width: `${(count / keys.length) * 100}%`,
                        background: "var(--brand-secondary)",
                      }}
                    />
                  </div>
                  <span className="text-xs font-medium w-8 text-right" style={{ color: "var(--text-primary)" }}>
                    {count}
                  </span>
                </div>
              ))}
          </div>
        </div>
      </div>

      {/* Recent keys */}
      <div
        className="p-5 rounded-xl"
        style={{ background: "var(--bg-surface)", border: "1px solid var(--border-default)" }}
      >
        <div className="flex items-center gap-2 mb-4">
          <TrendingUp size={16} style={{ color: "var(--text-muted)" }} />
          <h3 className="text-sm font-semibold" style={{ color: "var(--text-primary)" }}>
            最近更新
          </h3>
        </div>
        {recentKeys.length === 0 ? (
          <p className="text-sm text-center py-6" style={{ color: "var(--text-muted)" }}>
            暂无密钥
          </p>
        ) : (
          <div className="space-y-1">
            {recentKeys.map((key) => (
              <div
                key={key.id}
                className="flex items-center justify-between py-2.5 px-3 rounded-lg transition-smooth"
                style={{ color: "var(--text-secondary)" }}
              >
                <div className="flex items-center gap-3">
                  <KeyRound size={14} style={{ color: "var(--text-muted)" }} />
                  <span className="text-sm" style={{ color: "var(--text-primary)" }}>
                    {key.name}
                  </span>
                  <span className="text-xs" style={{ color: "var(--text-muted)" }}>
                    {key.provider}
                  </span>
                </div>
                <span className="text-xs" style={{ color: "var(--text-muted)" }}>
                  {new Date(key.updated_at).toLocaleDateString("zh-CN")}
                </span>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
