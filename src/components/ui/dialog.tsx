import * as React from 'react';

import { Dialog as BaseDialog } from '@base-ui/react/dialog';
import * as stylex from '@stylexjs/stylex';

import { ring } from '@/lib/stylex-utils';
import { duration, easing, fontSize, fontWeight, lineHeight, scale, space, z } from '@/lib/constants.stylex';
import { ink, radius } from '@/tokens.stylex';

interface StyleProp {
  style?: stylex.StyleXStyles;
}

export const Dialog = BaseDialog.Root;

export function DialogContent({ style, children, ...props }: Omit<React.ComponentPropsWithoutRef<typeof BaseDialog.Popup>, 'className' | 'style'> & StyleProp) {
  return (
    <BaseDialog.Portal>
      <BaseDialog.Backdrop {...stylex.props(styles.overlay)} />
      <BaseDialog.Popup {...props} {...stylex.props(styles.content, ring(), style)}>
        {children}
      </BaseDialog.Popup>
    </BaseDialog.Portal>
  );
}

export function DialogTitle({ style, ...props }: Omit<React.ComponentPropsWithoutRef<typeof BaseDialog.Title>, 'className' | 'style'> & StyleProp) {
  return <BaseDialog.Title {...props} {...stylex.props(styles.title, style)} />;
}

export function DialogDescription({ style, ...props }: Omit<React.ComponentPropsWithoutRef<typeof BaseDialog.Description>, 'className' | 'style'> & StyleProp) {
  return <BaseDialog.Description {...props} {...stylex.props(styles.description, style)} />;
}

export function DialogFooter({ style, ...props }: Omit<React.ComponentPropsWithoutRef<'div'>, 'className' | 'style'> & StyleProp) {
  return <div {...props} {...stylex.props(styles.footer, style)} />;
}

const styles = stylex.create({
  overlay: {
    backgroundColor: ink.overlay,
    inset: 0,
    opacity: {
      default: 1,
      '[data-starting-style]': 0,
      '[data-ending-style]': 0,
    },
    position: 'fixed',
    transitionDuration: duration.normal,
    transitionProperty: {
      default: 'opacity',
      '@media (prefers-reduced-motion: reduce)': 'none',
    },
    transitionTimingFunction: easing.out,
    zIndex: z.popup,
  },
  content: {
    backgroundColor: ink.ground,
    borderRadius: radius.window,
    color: ink.text,
    display: 'flex',
    flexDirection: 'column',
    gap: space.s12,
    height: 'fit-content',
    inset: 0,
    margin: 'auto',
    maxWidth: 512,
    opacity: {
      default: 1,
      '[data-starting-style]': 0,
      '[data-ending-style]': 0,
    },
    padding: space.s14,
    position: 'fixed',
    transform: {
      default: 'scale(1)',
      '[data-starting-style]': `scale(${scale.popupEnter})`,
      '[data-ending-style]': `scale(${scale.popupEnter})`,
    },
    transitionDuration: duration.normal,
    transitionProperty: {
      default: 'opacity, transform',
      '@media (prefers-reduced-motion: reduce)': 'none',
    },
    transitionTimingFunction: easing.out,
    width: `calc(100% - 2 * ${space.s28})`,
    zIndex: z.popup,
  },
  title: {
    fontSize: fontSize.heading,
    fontWeight: fontWeight.heading,
    lineHeight: lineHeight.heading,
  },
  description: {
    color: ink.textMuted,
    fontSize: fontSize.control,
    lineHeight: lineHeight.control,
    marginTop: `calc(-1 * ${space.s6})`,
  },
  footer: {
    display: 'flex',
    gap: space.s8,
    justifyContent: 'flex-end',
  },
});
