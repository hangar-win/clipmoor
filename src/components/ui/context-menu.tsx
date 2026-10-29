import * as React from 'react';

import { ContextMenu as BaseContextMenu } from '@base-ui/react/context-menu';
import * as stylex from '@stylexjs/stylex';
import { ChevronRight } from 'lucide-react';

import { icon, ring } from '@/lib/stylex-utils';
import { border, duration, easing, fontSize, lineHeight, opacity, scale, size, space, z } from '@/lib/constants.stylex';
import { ink, radius } from '@/tokens.stylex';

interface StyleProp {
  style?: stylex.StyleXStyles;
}

export const ContextMenu = BaseContextMenu.Root;
export const ContextMenuSub = BaseContextMenu.SubmenuRoot;

export function ContextMenuTrigger({ style, ...props }: Omit<React.ComponentPropsWithoutRef<typeof BaseContextMenu.Trigger>, 'className' | 'style'> & StyleProp) {
  return <BaseContextMenu.Trigger {...props} {...stylex.props(styles.trigger, style)} />;
}

export interface ContextMenuContentProps extends Omit<React.ComponentPropsWithoutRef<typeof BaseContextMenu.Popup>, 'className' | 'style'>, Pick<React.ComponentPropsWithoutRef<typeof BaseContextMenu.Positioner>, 'align' | 'alignOffset' | 'side' | 'sideOffset'>, StyleProp {}

export function ContextMenuContent({ style, side = 'right', sideOffset = 0, align = 'start', alignOffset = 4, ...props }: ContextMenuContentProps) {
  return (
    <BaseContextMenu.Portal>
      <BaseContextMenu.Positioner align={align} alignOffset={alignOffset} side={side} sideOffset={sideOffset} {...stylex.props(styles.positioner)}>
        <BaseContextMenu.Popup {...props} {...stylex.props(styles.popup, ring(), style)} />
      </BaseContextMenu.Positioner>
    </BaseContextMenu.Portal>
  );
}

export function ContextMenuItem({ style, ...props }: Omit<React.ComponentPropsWithoutRef<typeof BaseContextMenu.Item>, 'className' | 'style'> & StyleProp) {
  return <BaseContextMenu.Item {...props} {...stylex.props(styles.item, style)} />;
}

export function ContextMenuSubTrigger({ style, children, ...props }: Omit<React.ComponentPropsWithoutRef<typeof BaseContextMenu.SubmenuTrigger>, 'className' | 'style'> & StyleProp) {
  return (
    <BaseContextMenu.SubmenuTrigger {...props} {...stylex.props(styles.item, styles.subTrigger, style)}>
      {children}
      <ChevronRight {...stylex.props(icon.sm, styles.chevron)} />
    </BaseContextMenu.SubmenuTrigger>
  );
}

export function ContextMenuSubContent({ style, sideOffset = 0, alignOffset = -4, ...props }: ContextMenuContentProps) {
  return <ContextMenuContent side="right" sideOffset={sideOffset} alignOffset={alignOffset} {...props} style={[styles.subPopup, style]} />;
}

export function ContextMenuSeparator({ style, ...props }: Omit<React.ComponentPropsWithoutRef<typeof BaseContextMenu.Separator>, 'className' | 'style'> & StyleProp) {
  return <BaseContextMenu.Separator {...props} {...stylex.props(styles.separator, style)} />;
}

const styles = stylex.create({
  trigger: {
    userSelect: 'none',
  },
  positioner: {
    outline: 'none',
    zIndex: z.popup,
  },
  popup: {
    backgroundColor: ink.ground,
    borderRadius: radius.card,
    color: ink.text,
    maxHeight: 'var(--available-height)',
    maxWidth: size.menuMaxW,
    minWidth: size.menuMinW,
    opacity: {
      default: 1,
      '[data-starting-style]': 0,
      '[data-ending-style]': 0,
    },
    outline: 'none',
    overflowX: 'hidden',
    overflowY: 'auto',
    paddingBlock: space.s4,
    transform: {
      default: 'scale(1)',
      '[data-starting-style]': `scale(${scale.popupEnter})`,
      '[data-ending-style]': `scale(${scale.popupEnter})`,
    },
    transformOrigin: 'var(--transform-origin)',
    transitionDuration: duration.fast,
    transitionProperty: {
      default: 'opacity, transform',
      '@media (prefers-reduced-motion: reduce)': 'none',
    },
    transitionTimingFunction: easing.out,
  },
  subPopup: {
    width: 'max-content',
  },
  item: {
    alignItems: 'center',
    backgroundColor: {
      default: 'transparent',
      '[data-highlighted]': ink.strip,
    },
    borderRadius: radius.item,
    color: {
      default: null,
      '[data-disabled]': ink.textMuted,
    },
    cursor: 'default',
    display: 'flex',
    fontSize: fontSize.title,
    gap: space.s8,
    lineHeight: lineHeight.title,
    marginInline: space.s4,
    opacity: { default: 1, '[data-disabled]': opacity.disabled },
    outline: 'none',
    paddingBlock: space.s5,
    paddingInline: space.s10,
    userSelect: 'none',
  },
  subTrigger: {
    backgroundColor: {
      default: 'transparent',
      '[data-highlighted]': ink.strip,
      '[data-popup-open]': ink.strip,
    },
  },
  chevron: {
    color: ink.textMuted,
    marginLeft: 'auto',
  },
  separator: {
    backgroundColor: ink.hairline,
    height: border.stroke,
    marginBlock: space.s4,
  },
});
export { styles as menuStyles };
