import { useQuery } from "@tanstack/react-query";
import { listKeys, listGroups } from "@/api";

export function DashboardPage() {
  const { data: keys = [] } = useQuery({
    queryKey: ["keys"],
    queryFn: listKeys,
  });

  const { data: groups = [] } = useQuery({
    queryKey: ["groups"],
    queryFn: listGroups,
  });

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

  return (
    <div className="space-y-6">
      <h2 className="text-2xl font-bold text-vault-text">仪表盘</h2>

      {/* 统计卡片 */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div className="bg-vault-surface rounded-lg p-6 border border-vault-border">
          <div className="text-3xl font-bold text-vault-primary">{keys.length}</div>
          <div className="text-vault-muted mt-1">总密钥数</div>
        </div>
        <div className="bg-vault-surface rounded-lg p-6 border border-vault-border">
          <div className="text-3xl font-bold text-vault-success">{groups.length}</div>
          <div className="text-vault-muted mt-1">分组数</div>
        </div>
        <div className="bg-vault-surface rounded-lg p-6 border border-vault-border">
          <div className="text-3xl font-bold text-vault-warning">
            {Object.keys(envStats).length}
          </div>
          <div className="text-vault-muted mt-1">环境数</div>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* 环境分布 */}
        <div className="bg-vault-surface rounded-lg p-6 border border-vault-border">
          <h3 className="text-lg font-semibold text-vault-text mb-4">环境分布</h3>
          <div className="space-y-3">
            {Object.entries(envStats).map(([env, count]) => (
              <div key={env} className="flex items-center justify-between">
                <span className="text-vault-muted">{env}</span>
                <div className="flex items-center space-x-3">
                  <div className="w-32 bg-vault-bg rounded-full h-2">
                    <div
                      className="bg-vault-primary rounded-full h-2"
                      style={{ width: `${(count / keys.length) * 100}%` }}
                    />
                  </div>
                  <span className="text-vault-text font-medium w-8 text-right">
                    {count}
                  </span>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* 提供商分布 */}
        <div className="bg-vault-surface rounded-lg p-6 border border-vault-border">
          <h3 className="text-lg font-semibold text-vault-text mb-4">提供商分布</h3>
          <div className="space-y-3">
            {Object.entries(providerStats)
              .sort(([, a], [, b]) => b - a)
              .slice(0, 5)
              .map(([provider, count]) => (
                <div key={provider} className="flex items-center justify-between">
                  <span className="text-vault-muted">{provider}</span>
                  <div className="flex items-center space-x-3">
                    <div className="w-32 bg-vault-bg rounded-full h-2">
                      <div
                        className="bg-vault-success rounded-full h-2"
                        style={{ width: `${(count / keys.length) * 100}%` }}
                      />
                    </div>
                    <span className="text-vault-text font-medium w-8 text-right">
                      {count}
                    </span>
                  </div>
                </div>
              ))}
          </div>
        </div>
      </div>

      {/* 最近活动 */}
      <div className="bg-vault-surface rounded-lg p-6 border border-vault-border">
        <h3 className="text-lg font-semibold text-vault-text mb-4">最近更新的密钥</h3>
        {recentKeys.length === 0 ? (
          <p className="text-vault-muted text-center py-4">暂无密钥</p>
        ) : (
          <div className="space-y-3">
            {recentKeys.map((key) => (
              <div
                key={key.id}
                className="flex items-center justify-between py-2 border-b border-vault-border last:border-0"
              >
                <div>
                  <span className="text-vault-text font-medium">{key.name}</span>
                  <span className="text-vault-muted text-sm ml-2">{key.provider}</span>
                </div>
                <span className="text-vault-muted text-sm">
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
