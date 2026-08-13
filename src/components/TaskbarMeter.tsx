import { useEffect, useState, type CSSProperties } from "react";
import type { TaskbarAppearance } from "../lib/settings";
import type { MeterDisplayItem, ProviderUsageStatus } from "../lib/usage";

type TaskbarMeterProps = {
  label: string;
  resetLabel?: string | null;
  remainingPercent: number;
  valueLabel?: string | null;
  items?: MeterDisplayItem[];
  scrollSeconds?: number;
  scrollAnimationSeconds?: number;
  appearance?: TaskbarAppearance;
  status: ProviderUsageStatus;
  loading: boolean;
};

export function TaskbarMeter({
  label,
  resetLabel,
  remainingPercent,
  valueLabel,
  items,
  scrollSeconds = 3.2,
  scrollAnimationSeconds = 0.35,
  appearance,
  status,
  loading,
}: TaskbarMeterProps) {
  const carouselItems = items?.length ? items : null;
  const appearanceStyle = meterAppearanceStyle(appearance);
  const progressColor = appearance?.progressColor;
  const [activeItemIndex, setActiveItemIndex] = useState(0);
  const [animateScroll, setAnimateScroll] = useState(true);

  useEffect(() => {
    if (!carouselItems || carouselItems.length <= 1) {
      setActiveItemIndex(0);
      return;
    }

    const safeHoldMs = Math.max(0.5, scrollSeconds) * 1000;
    const timeout = window.setTimeout(() => {
      setAnimateScroll(true);
      setActiveItemIndex((current) => current + 1);
    }, safeHoldMs);

    return () => window.clearTimeout(timeout);
  }, [activeItemIndex, carouselItems, scrollSeconds]);

  useEffect(() => {
    if (!carouselItems || activeItemIndex < carouselItems.length) {
      return;
    }

    const safeAnimationMs = Math.max(0.05, scrollAnimationSeconds) * 1000;
    const timeout = window.setTimeout(() => {
      setAnimateScroll(false);
      setActiveItemIndex(0);
      window.requestAnimationFrame(() => {
        window.requestAnimationFrame(() => setAnimateScroll(true));
      });
    }, safeAnimationMs);

    return () => window.clearTimeout(timeout);
  }, [activeItemIndex, carouselItems, scrollAnimationSeconds]);

  if (carouselItems && carouselItems.length > 1) {
    const safeAnimationSeconds = Number.isFinite(scrollAnimationSeconds)
      ? Math.max(0.05, scrollAnimationSeconds)
      : 0.35;
    return (
      <div
        className="taskbar-meter is-scrolling"
        style={
          {
            ...appearanceStyle,
            "--meter-item-count": carouselItems.length,
            "--meter-scroll-transition": animateScroll
              ? `transform ${safeAnimationSeconds}s ease-in-out`
              : "none",
            "--meter-scroll-offset": activeItemIndex,
          } as CSSProperties
        }
      >
        <div className="meter-carousel">
          <div className="meter-carousel-track">
            {carouselItems.map((item) => (
              <MeterRow
                key={item.id}
                label={item.label}
                loading={loading}
                remainingPercent={item.remainingPercent}
                resetLabel={item.resetLabel}
                status={item.status}
                valueLabel={item.valueLabel}
                progressColor={progressColor}
              />
            ))}
            <MeterRow
              label={carouselItems[0].label}
              loading={loading}
              remainingPercent={carouselItems[0].remainingPercent}
              resetLabel={carouselItems[0].resetLabel}
              status={carouselItems[0].status}
              valueLabel={carouselItems[0].valueLabel}
              progressColor={progressColor}
            />
          </div>
        </div>
      </div>
    );
  }

  const safePercent = Math.max(0, Math.min(100, remainingPercent));
  const tone = status !== "ok" ? "neutral" : safePercent <= 10 ? "danger" : safePercent <= 30 ? "warn" : "good";
  const displayValue = valueLabel ?? `${safePercent}%`;

  return (
    <div className="taskbar-meter" data-tone={tone} style={appearanceStyle}>
      <div className="meter-meta">
        <span className="meter-label">{label}</span>
        <span className="meter-value">{loading ? "..." : displayValue}</span>
      </div>
      <div className="meter-track" aria-hidden="true">
        <div className="meter-fill" style={{ backgroundColor: progressColor, width: `${safePercent}%` }} />
      </div>
      {resetLabel && <span className="meter-reset">{resetLabel}</span>}
    </div>
  );
}

function MeterRow({
  label,
  remainingPercent,
  valueLabel,
  resetLabel,
  status,
  loading,
  progressColor,
}: {
  label: string;
  remainingPercent: number;
  valueLabel?: string | null;
  resetLabel?: string | null;
  status: ProviderUsageStatus;
  loading: boolean;
  progressColor?: string;
}) {
  const safePercent = Math.max(0, Math.min(100, remainingPercent));
  const tone = status !== "ok" ? "neutral" : safePercent <= 10 ? "danger" : safePercent <= 30 ? "warn" : "good";
  const displayValue = valueLabel ?? `${safePercent}%`;

  return (
    <div className="meter-carousel-item" data-tone={tone}>
      <div className="meter-meta">
        <span className="meter-label">{label}</span>
        <span className="meter-value">{loading ? "..." : displayValue}</span>
      </div>
      <div className="meter-track" aria-hidden="true">
        <div className="meter-fill" style={{ backgroundColor: progressColor, width: `${safePercent}%` }} />
      </div>
      {resetLabel && <span className="meter-reset">{resetLabel}</span>}
    </div>
  );
}

function meterAppearanceStyle(appearance?: TaskbarAppearance): CSSProperties {
  if (!appearance) {
    return {};
  }

  return {
    "--meter-text-size": `${appearance.textSizePx}px`,
    "--meter-reset-text-size": `${appearance.resetTextSizePx}px`,
    "--meter-text-color": appearance.textColor,
  } as CSSProperties;
}
