import type { SkillOrAgentInfo } from "../../lib/commands";

function List({ title, items }: { title: string; items: SkillOrAgentInfo[] }) {
  return (
    <section>
      <h3 className="mb-2 font-medium text-neutral-200">{title}</h3>
      {items.length === 0 ? (
        <p className="text-sm text-neutral-500">Nenhum.</p>
      ) : (
        <ul className="divide-y divide-neutral-800 text-sm">
          {items.map((item) => (
            <li key={item.id} className="flex items-center justify-between py-2">
              <div>
                <p className="text-neutral-200">{item.name}</p>
                <p className="text-xs text-neutral-500">{item.description}</p>
              </div>
              {item.builtIn && <span className="text-xs text-neutral-500">herdado</span>}
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}

export function SkillsAgentsTab({ skills, agents }: { skills: SkillOrAgentInfo[]; agents: SkillOrAgentInfo[] }) {
  return (
    <div className="space-y-6 p-6">
      <List title="Skills" items={skills} />
      <List title="Agents" items={agents} />
    </div>
  );
}
