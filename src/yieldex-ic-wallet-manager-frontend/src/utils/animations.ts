// Framer Motion animation configurations and utilities

import { Variants, Transition } from 'framer-motion';

// Easing functions
export const easing = {
  easeInOut: [0.4, 0, 0.2, 1] as const,
  easeOut: [0, 0, 0.2, 1] as const,
  easeIn: [0.4, 0, 1, 1] as const,
  bounce: [0.68, -0.55, 0.265, 1.55] as const,
  spring: { type: 'spring', damping: 20, stiffness: 300 } as const,
  gentle: { type: 'spring', damping: 25, stiffness: 120 } as const,
};

// Common transitions
export const transitions: Record<string, Transition> = {
  smooth: {
    duration: 0.3,
    ease: easing.easeInOut
  },
  gentle: {
    duration: 0.4,
    ease: easing.easeOut
  },
  bouncy: {
    ...easing.spring,
    duration: 0.5
  },
  slow: {
    duration: 0.6,
    ease: easing.easeInOut
  }
};

// Page transitions
export const pageVariants: Variants = {
  initial: {
    opacity: 0,
    y: 20,
    scale: 0.98
  },
  enter: {
    opacity: 1,
    y: 0,
    scale: 1,
    transition: transitions.gentle
  },
  exit: {
    opacity: 0,
    y: -20,
    scale: 1.02,
    transition: transitions.smooth
  }
};

// Card animations
export const cardVariants: Variants = {
  initial: {
    opacity: 0,
    y: 30,
    scale: 0.95
  },
  animate: {
    opacity: 1,
    y: 0,
    scale: 1,
    transition: transitions.gentle
  },
  hover: {
    y: -4,
    scale: 1.02,
    transition: transitions.smooth
  },
  tap: {
    scale: 0.98,
    transition: { duration: 0.1 }
  }
};

// List item animations
export const listVariants: Variants = {
  hidden: {
    opacity: 0
  },
  visible: {
    opacity: 1,
    transition: {
      staggerChildren: 0.1,
      delayChildren: 0.1
    }
  }
};

export const listItemVariants: Variants = {
  hidden: {
    opacity: 0,
    x: -20,
    y: 10
  },
  visible: {
    opacity: 1,
    x: 0,
    y: 0,
    transition: transitions.gentle
  }
};

// Button animations
export const buttonVariants: Variants = {
  rest: {
    scale: 1
  },
  hover: {
    scale: 1.05,
    transition: transitions.smooth
  },
  tap: {
    scale: 0.95,
    transition: { duration: 0.1 }
  },
  loading: {
    scale: [1, 1.02, 1],
    transition: {
      duration: 1.5,
      repeat: Infinity,
      ease: easing.easeInOut
    }
  }
};

// Modal animations
export const modalVariants: Variants = {
  hidden: {
    opacity: 0,
    scale: 0.8,
    y: 50
  },
  visible: {
    opacity: 1,
    scale: 1,
    y: 0,
    transition: {
      ...transitions.bouncy,
      duration: 0.4
    }
  },
  exit: {
    opacity: 0,
    scale: 0.8,
    y: 50,
    transition: transitions.smooth
  }
};

// Backdrop animations
export const backdropVariants: Variants = {
  hidden: { opacity: 0 },
  visible: { 
    opacity: 1,
    transition: transitions.smooth
  },
  exit: { 
    opacity: 0,
    transition: transitions.smooth
  }
};

// Number counter animations
export const counterVariants: Variants = {
  initial: {
    opacity: 0,
    y: 20
  },
  animate: {
    opacity: 1,
    y: 0,
    transition: {
      duration: 0.8,
      ease: easing.easeOut
    }
  }
};

// Loading spinner
export const spinnerVariants: Variants = {
  animate: {
    rotate: 360,
    transition: {
      duration: 1,
      repeat: Infinity,
      ease: 'linear'
    }
  }
};

// Pulse animation
export const pulseVariants: Variants = {
  animate: {
    scale: [1, 1.05, 1],
    opacity: [0.7, 1, 0.7],
    transition: {
      duration: 2,
      repeat: Infinity,
      ease: easing.easeInOut
    }
  }
};

// Slide animations - separate variants for each direction
export const slideInLeftVariants: Variants = {
  initial: { x: -100, opacity: 0 },
  animate: { x: 0, opacity: 1 },
  exit: { x: -100, opacity: 0 }
};

export const slideInRightVariants: Variants = {
  initial: { x: 100, opacity: 0 },
  animate: { x: 0, opacity: 1 },
  exit: { x: 100, opacity: 0 }
};

export const slideInUpVariants: Variants = {
  initial: { y: 100, opacity: 0 },
  animate: { y: 0, opacity: 1 },
  exit: { y: 100, opacity: 0 }
};

export const slideInDownVariants: Variants = {
  initial: { y: -100, opacity: 0 },
  animate: { y: 0, opacity: 1 },
  exit: { y: -100, opacity: 0 }
};

// For backwards compatibility
export const slideVariants = {
  slideInLeft: slideInLeftVariants,
  slideInRight: slideInRightVariants,
  slideInUp: slideInUpVariants,
  slideInDown: slideInDownVariants
};

// Fade animations
export const fadeVariants: Variants = {
  initial: { opacity: 0 },
  animate: { 
    opacity: 1,
    transition: transitions.gentle
  },
  exit: { 
    opacity: 0,
    transition: transitions.smooth
  }
};

// Stagger container for multiple items
export const staggerContainer: Variants = {
  hidden: {},
  visible: {
    transition: {
      staggerChildren: 0.1,
      delayChildren: 0.1
    }
  }
};

// Success/Error animations
export const statusVariants: Variants = {
  success: {
    scale: [1, 1.2, 1],
    transition: {
      duration: 0.5,
      ease: easing.bounce
    }
  },
  error: {
    x: [-4, 4, -4, 4, 0],
    transition: {
      duration: 0.4,
      ease: easing.easeInOut
    }
  }
};

// Chart/Graph animations
export const chartVariants: Variants = {
  initial: {
    pathLength: 0,
    opacity: 0
  },
  animate: {
    pathLength: 1,
    opacity: 1,
    transition: {
      pathLength: { duration: 2, ease: easing.easeInOut },
      opacity: { duration: 0.3 }
    }
  }
};

// Utility functions
export const getStaggeredDelay = (index: number, baseDelay: number = 0.1): number => {
  return index * baseDelay;
};

export const createStaggerVariants = (staggerDelay: number = 0.1): Variants => ({
  hidden: { opacity: 0, y: 20 },
  visible: (index: number) => ({
    opacity: 1,
    y: 0,
    transition: {
      delay: index * staggerDelay,
      ...transitions.gentle
    }
  })
});

// Layout animations for reordering
export const layoutTransition: Transition = {
  type: 'spring',
  damping: 20,
  stiffness: 300
};

// Custom animation hooks utilities
export const useCounterAnimation = (
  value: number,
  duration: number = 2000
) => {
  // This would be implemented as a custom hook in a real component
  // For now, we provide the configuration
  return {
    initial: 0,
    animate: value,
    transition: {
      duration: duration / 1000,
      ease: easing.easeOut
    }
  };
};

// Animation presets for common UI patterns
export const presets = {
  cardHover: {
    whileHover: cardVariants.hover,
    whileTap: cardVariants.tap
  },
  buttonPress: {
    whileHover: buttonVariants.hover,
    whileTap: buttonVariants.tap
  },
  fadeInUp: {
    initial: { opacity: 0, y: 30 },
    animate: { opacity: 1, y: 0 },
    transition: transitions.gentle
  },
  scaleIn: {
    initial: { scale: 0.8, opacity: 0 },
    animate: { scale: 1, opacity: 1 },
    transition: transitions.bouncy
  }
};

// Performance optimization: reduce motion for users who prefer it
export const respectsReducedMotion = (animation: any) => {
  if (typeof window !== 'undefined' && window.matchMedia) {
    const prefersReducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    if (prefersReducedMotion) {
      return {
        ...animation,
        transition: { duration: 0.01 }
      };
    }
  }
  return animation;
};