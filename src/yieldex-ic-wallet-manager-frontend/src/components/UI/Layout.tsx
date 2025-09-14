import React from 'react';
import { motion } from 'framer-motion';
import { pageVariants, transitions } from '@/utils/animations';
import { clsx } from 'clsx';

export interface LayoutProps {
  children: React.ReactNode;
  className?: string;
  maxWidth?: 'sm' | 'md' | 'lg' | 'xl' | '2xl' | 'full';
  padding?: boolean;
  centered?: boolean;
}

const Layout: React.FC<LayoutProps> = ({
  children,
  className,
  maxWidth = 'full',
  padding = true,
  centered = false
}) => {
  const maxWidthClasses = {
    sm: 'max-w-sm',
    md: 'max-w-md',
    lg: 'max-w-4xl',
    xl: 'max-w-6xl',
    '2xl': 'max-w-7xl',
    full: 'max-w-full'
  };

  return (
    <div className={clsx(
      'min-h-screen bg-gray-900',
      centered && 'flex items-center justify-center'
    )}>
      <div className={clsx(
        'w-full mx-auto',
        maxWidthClasses[maxWidth],
        padding && 'px-4 sm:px-6 lg:px-8',
        !centered && 'py-8',
        className
      )}>
        <motion.div
          variants={pageVariants}
          initial="initial"
          animate="enter"
          exit="exit"
        >
          {children}
        </motion.div>
      </div>
    </div>
  );
};

// Header component
export const Header: React.FC<{
  children: React.ReactNode;
  className?: string;
}> = ({ children, className }) => (
  <motion.header 
    className={clsx('mb-8', className)}
    initial={{ opacity: 0, y: -20 }}
    animate={{ opacity: 1, y: 0 }}
    transition={transitions.gentle}
  >
    {children}
  </motion.header>
);

// Main content area
export const Main: React.FC<{
  children: React.ReactNode;
  className?: string;
}> = ({ children, className }) => (
  <motion.main 
    className={clsx('flex-1', className)}
    initial={{ opacity: 0 }}
    animate={{ opacity: 1 }}
    transition={{ ...transitions.gentle, delay: 0.1 }}
  >
    {children}
  </motion.main>
);

// Navigation component
export const Navigation: React.FC<{
  children: React.ReactNode;
  className?: string;
}> = ({ children, className }) => (
  <motion.nav 
    className={clsx('mb-6', className)}
    initial={{ opacity: 0, x: -20 }}
    animate={{ opacity: 1, x: 0 }}
    transition={transitions.gentle}
  >
    {children}
  </motion.nav>
);

// Section component
export const Section: React.FC<{
  children: React.ReactNode;
  title?: string;
  description?: string;
  className?: string;
  headerClassName?: string;
}> = ({ children, title, description, className, headerClassName }) => (
  <motion.section 
    className={clsx('mb-8', className)}
    initial={{ opacity: 0, y: 20 }}
    animate={{ opacity: 1, y: 0 }}
    transition={transitions.gentle}
  >
    {(title || description) && (
      <div className={clsx('mb-6', headerClassName)}>
        {title && (
          <motion.h2 
            className="text-2xl font-bold text-white mb-2"
            initial={{ opacity: 0, x: -20 }}
            animate={{ opacity: 1, x: 0 }}
            transition={transitions.gentle}
          >
            {title}
          </motion.h2>
        )}
        {description && (
          <motion.p 
            className="text-gray-400"
            initial={{ opacity: 0, x: -20 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ ...transitions.gentle, delay: 0.1 }}
          >
            {description}
          </motion.p>
        )}
      </div>
    )}
    {children}
  </motion.section>
);

// Grid layout for cards
export const Grid: React.FC<{
  children: React.ReactNode;
  cols?: 1 | 2 | 3 | 4 | 6 | 12;
  gap?: 'sm' | 'md' | 'lg' | 'xl';
  className?: string;
}> = ({ children, cols = 3, gap = 'md', className }) => {
  const colsClasses = {
    1: 'grid-cols-1',
    2: 'grid-cols-1 md:grid-cols-2',
    3: 'grid-cols-1 md:grid-cols-2 lg:grid-cols-3',
    4: 'grid-cols-1 md:grid-cols-2 lg:grid-cols-4',
    6: 'grid-cols-1 md:grid-cols-3 lg:grid-cols-6',
    12: 'grid-cols-12'
  };

  const gapClasses = {
    sm: 'gap-4',
    md: 'gap-6',
    lg: 'gap-8',
    xl: 'gap-12'
  };

  return (
    <div className={clsx(
      'grid',
      colsClasses[cols],
      gapClasses[gap],
      className
    )}>
      {children}
    </div>
  );
};

// Flex layout
export const Flex: React.FC<{
  children: React.ReactNode;
  direction?: 'row' | 'col';
  align?: 'start' | 'center' | 'end' | 'stretch';
  justify?: 'start' | 'center' | 'end' | 'between' | 'around' | 'evenly';
  wrap?: boolean;
  gap?: 'sm' | 'md' | 'lg' | 'xl';
  className?: string;
}> = ({ 
  children, 
  direction = 'row', 
  align = 'start', 
  justify = 'start', 
  wrap = false,
  gap = 'md',
  className 
}) => {
  const directionClasses = {
    row: 'flex-row',
    col: 'flex-col'
  };

  const alignClasses = {
    start: 'items-start',
    center: 'items-center',
    end: 'items-end',
    stretch: 'items-stretch'
  };

  const justifyClasses = {
    start: 'justify-start',
    center: 'justify-center',
    end: 'justify-end',
    between: 'justify-between',
    around: 'justify-around',
    evenly: 'justify-evenly'
  };

  const gapClasses = {
    sm: 'gap-2',
    md: 'gap-4',
    lg: 'gap-6',
    xl: 'gap-8'
  };

  return (
    <div className={clsx(
      'flex',
      directionClasses[direction],
      alignClasses[align],
      justifyClasses[justify],
      wrap && 'flex-wrap',
      gapClasses[gap],
      className
    )}>
      {children}
    </div>
  );
};

// Container for centering content
export const Container: React.FC<{
  children: React.ReactNode;
  size?: 'sm' | 'md' | 'lg' | 'xl' | 'full';
  className?: string;
}> = ({ children, size = 'lg', className }) => {
  const sizeClasses = {
    sm: 'max-w-2xl',
    md: 'max-w-4xl',
    lg: 'max-w-6xl',
    xl: 'max-w-7xl',
    full: 'max-w-full'
  };

  return (
    <div className={clsx(
      'mx-auto px-4 sm:px-6 lg:px-8',
      sizeClasses[size],
      className
    )}>
      {children}
    </div>
  );
};

// Loading skeleton
export const Skeleton: React.FC<{
  className?: string;
  height?: string;
  width?: string;
  rounded?: boolean;
}> = ({ className, height = 'h-4', width = 'w-full', rounded = false }) => (
  <div className={clsx(
    'animate-pulse bg-gray-700/50',
    height,
    width,
    rounded ? 'rounded-full' : 'rounded',
    className
  )} />
);

// Empty state
export const EmptyState: React.FC<{
  title: string;
  description?: string;
  action?: React.ReactNode;
  icon?: React.ReactNode;
  className?: string;
}> = ({ title, description, action, icon, className }) => (
  <motion.div 
    className={clsx(
      'flex flex-col items-center justify-center py-12 px-4 text-center',
      className
    )}
    initial={{ opacity: 0, scale: 0.9 }}
    animate={{ opacity: 1, scale: 1 }}
    transition={transitions.gentle}
  >
    {icon && (
      <div className="text-gray-500 mb-4">
        {icon}
      </div>
    )}
    <h3 className="text-lg font-semibold text-white mb-2">{title}</h3>
    {description && (
      <p className="text-gray-400 mb-4 max-w-md">{description}</p>
    )}
    {action}
  </motion.div>
);

export default Layout;