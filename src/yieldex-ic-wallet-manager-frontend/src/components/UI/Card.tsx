import React from 'react';
import { motion } from 'framer-motion';
import { cardVariants, transitions } from '@/utils/animations';
import { clsx } from 'clsx';

export interface CardProps extends React.HTMLAttributes<HTMLDivElement> {
  variant?: 'default' | 'glass' | 'gradient' | 'outline';
  padding?: 'none' | 'sm' | 'md' | 'lg' | 'xl';
  hoverable?: boolean;
  clickable?: boolean;
  children: React.ReactNode;
}

const Card = React.forwardRef<HTMLDivElement, CardProps>(
  ({
    className,
    variant = 'default',
    padding = 'md',
    hoverable = true,
    clickable = false,
    children,
    onClick,
    ...props
  }, ref) => {

    const baseClasses = clsx(
      'rounded-xl border transition-all duration-300',
      {
        'cursor-pointer': clickable || onClick,
        'cursor-default': !clickable && !onClick
      }
    );

    const variantClasses = {
      default: clsx(
        'bg-gray-800/50 border-gray-700/50',
        'hover:bg-gray-800/70 hover:border-gray-600/50'
      ),
      glass: clsx(
        'bg-gray-800/20 backdrop-blur-xl border-gray-700/50',
        'hover:bg-gray-800/30 hover:border-gray-600/50',
        'shadow-2xl'
      ),
      gradient: clsx(
        'bg-gradient-to-br from-gray-800/50 to-gray-900/50 border-gray-700/50',
        'hover:from-gray-800/70 hover:to-gray-900/70',
        'shadow-xl'
      ),
      outline: clsx(
        'bg-transparent border-gray-600',
        'hover:bg-gray-800/30 hover:border-gray-500'
      )
    };

    const paddingClasses = {
      none: '',
      sm: 'p-3',
      md: 'p-4',
      lg: 'p-6',
      xl: 'p-8'
    };

    const isInteractive = clickable || hoverable || onClick;

    const motionProps = isInteractive ? {
      variants: cardVariants,
      initial: "initial",
      animate: "animate",
      whileHover: hoverable ? "hover" : undefined,
      whileTap: (clickable || onClick) ? "tap" : undefined,
      layout: true,
      transition: transitions.gentle
    } : {
      initial: { opacity: 0, y: 20 },
      animate: { opacity: 1, y: 0 },
      transition: transitions.gentle
    };

    // Separate HTML div props from motion props to avoid conflicts
    const { 
      onAnimationStart, onAnimationEnd, onAnimationIteration,
      onTransitionEnd, onDrag, onDragEnd, onDragStart,
      ...htmlProps 
    } = props;

    return (
      <motion.div
        ref={ref}
        className={clsx(
          baseClasses,
          variantClasses[variant],
          paddingClasses[padding],
          className
        )}
        onClick={onClick}
        {...motionProps}
        {...htmlProps}
      >
        {children}
      </motion.div>
    );
  }
);

Card.displayName = 'Card';

// Card header component
export const CardHeader: React.FC<{
  children: React.ReactNode;
  className?: string;
}> = ({ children, className }) => (
  <div className={clsx('flex items-center justify-between mb-4', className)}>
    {children}
  </div>
);

// Card title component
export const CardTitle: React.FC<{
  children: React.ReactNode;
  className?: string;
  size?: 'sm' | 'md' | 'lg' | 'xl';
}> = ({ children, className, size = 'lg' }) => {
  const sizeClasses = {
    sm: 'text-sm font-medium',
    md: 'text-base font-semibold',
    lg: 'text-lg font-semibold',
    xl: 'text-xl font-bold'
  };

  return (
    <h3 className={clsx('text-white', sizeClasses[size], className)}>
      {children}
    </h3>
  );
};

// Card content component
export const CardContent: React.FC<{
  children: React.ReactNode;
  className?: string;
}> = ({ children, className }) => (
  <div className={clsx('text-gray-300', className)}>
    {children}
  </div>
);

// Card footer component
export const CardFooter: React.FC<{
  children: React.ReactNode;
  className?: string;
}> = ({ children, className }) => (
  <div className={clsx('mt-4 pt-4 border-t border-gray-700/50 flex items-center justify-between', className)}>
    {children}
  </div>
);

// Metric card for displaying stats
export const MetricCard: React.FC<{
  label: string;
  value: string;
  change?: string;
  changeType?: 'positive' | 'negative' | 'neutral';
  icon?: React.ReactNode;
  className?: string;
}> = ({ label, value, change, changeType = 'neutral', icon, className }) => {
  const changeColors = {
    positive: 'text-green-400',
    negative: 'text-red-400',
    neutral: 'text-gray-400'
  };

  return (
    <Card variant="glass" className={clsx('metric-card', className)}>
      <div className="flex items-start justify-between">
        <div className="flex-1">
          <p className="text-sm text-gray-400 mb-1">{label}</p>
          <motion.p 
            className="text-2xl font-bold text-white mb-1"
            initial={{ opacity: 0, scale: 0.8 }}
            animate={{ opacity: 1, scale: 1 }}
            transition={transitions.gentle}
          >
            {value}
          </motion.p>
          {change && (
            <motion.p 
              className={clsx('text-sm', changeColors[changeType])}
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ ...transitions.gentle, delay: 0.1 }}
            >
              {change}
            </motion.p>
          )}
        </div>
        {icon && (
          <motion.div 
            className="text-primary-400 opacity-60"
            initial={{ opacity: 0, rotate: -10 }}
            animate={{ opacity: 0.6, rotate: 0 }}
            transition={transitions.gentle}
          >
            {icon}
          </motion.div>
        )}
      </div>
    </Card>
  );
};

// Protocol card for strategy display
export const ProtocolCard: React.FC<{
  name: string;
  apy: number;
  tvl: string;
  logo?: string;
  risk: 'conservative' | 'moderate' | 'aggressive';
  allocation?: number;
  className?: string;
  onClick?: () => void;
}> = ({ name, apy, tvl, logo, risk, allocation, className, onClick }) => {
  const riskColors = {
    conservative: 'text-green-400 bg-green-400/10',
    moderate: 'text-yellow-400 bg-yellow-400/10',
    aggressive: 'text-red-400 bg-red-400/10'
  };

  return (
    <Card 
      variant="glass" 
      className={clsx('cursor-pointer', className)}
      clickable
      onClick={onClick}
    >
      <div className="flex items-center space-x-4">
        {logo && (
          <div className="w-10 h-10 rounded-full bg-gray-700/50 flex items-center justify-center">
            <img src={logo} alt={name} className="w-6 h-6" />
          </div>
        )}
        <div className="flex-1">
          <div className="flex items-center justify-between mb-1">
            <h4 className="font-semibold text-white">{name}</h4>
            <span className="text-lg font-bold text-primary-400">{apy.toFixed(2)}%</span>
          </div>
          <div className="flex items-center justify-between text-sm">
            <span className="text-gray-400">TVL: {tvl}</span>
            <div className="flex items-center space-x-2">
              <span className={clsx('px-2 py-1 rounded text-xs font-medium', riskColors[risk])}>
                {risk}
              </span>
              {allocation && (
                <span className="text-gray-400">{allocation}%</span>
              )}
            </div>
          </div>
        </div>
      </div>
    </Card>
  );
};

export default Card;