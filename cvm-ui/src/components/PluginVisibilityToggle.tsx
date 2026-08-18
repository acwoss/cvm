interface Props {
  checked: boolean;
  onChange: (checked: boolean) => void;
}

export function PluginVisibilityToggle({ checked, onChange }: Props) {
  return (
    <label className="flex items-center gap-2 text-xs text-neutral-400">
      <button
        type="button"
        onClick={() => onChange(!checked)}
        className="relative flex h-[18px] w-8 flex-shrink-0 items-center justify-center rounded-full transition-colors"
        style={{ background: checked ? "rgba(0,229,255,0.18)" : "#282B33" }}
      >
        <span
          className="absolute top-0.5 h-3.5 w-3.5 rounded-full transition-all"
          style={{ left: checked ? 15 : 2, background: checked ? "#00E5FF" : "#555B68" }}
        />
      </button>
      Show plugin items
    </label>
  );
}
