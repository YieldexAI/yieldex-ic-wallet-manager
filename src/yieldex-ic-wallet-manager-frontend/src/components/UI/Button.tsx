import React from 'react';
import { motion, HTMLMotionProps } from 'framer-motion';
import { buttonVariants, transitions } from '@/utils/animations';
import { clsx } from 'clsx';

export interface ButtonProps extends Omit<HTMLMotionProps<'button'>, 'ref'> {
  variant?: 'primary' | 'secondary' | 'outline' | 'ghost' | 'danger';
  size?: 'sm' | 'md' | 'lg' | 'xl';
  loading?: boolean;
  fullWidth?: boolean;
  leftIcon?: React.ReactNode;
  rightIcon?: React.ReactNode;
}

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({
    className,
    variant = 'primary',
    size = 'md',
    loading = false,
    fullWidth = false,
    leftIcon,
    rightIcon,
    children,
    disabled,
    ...props
  }, ref) => {
    
    const baseClasses = clsx(
      'inline-flex items-center justify-center font-medium rounded-lg',
      'focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-gray-900',
      'transition-colors duration-200',
      'disabled:opacity-50 disabled:cursor-not-allowed disabled:pointer-events-none',
      {
        'w-full': fullWidth,
        'cursor-not-allowed': loading
      }
    );

    const variantClasses = {
      primary: clsx(
        'bg-gradient-to-r from-primary-600 to-primary-700 text-white',
        'hover:from-primary-500 hover:to-primary-600',
        'focus:ring-primary-500',
        'shadow-lg hover:shadow-primary-500/25'
      ),
      secondary: clsx(
        'bg-gray-700 text-gray-100',
        'hover:bg-gray-600',
        'focus:ring-gray-500'
      ),
      outline: clsx(
        'border border-gray-600 text-gray-300 bg-transparent',
        'hover:bg-gray-800 hover:border-gray-500',
        'focus:ring-gray-500'
      ),
      ghost: clsx(
        'text-gray-300 bg-transparent',
        'hover:bg-gray-800',
        'focus:ring-gray-500'
      ),
      danger: clsx(
        'bg-gradient-to-r from-red-600 to-red-700 text-white',
        'hover:from-red-500 hover:to-red-600',
        'focus:ring-red-500',
        'shadow-lg hover:shadow-red-500/25'
      )
    };

    const sizeClasses = {
      sm: 'px-3 py-1.5 text-sm',
      md: 'px-4 py-2 text-sm',
      lg: 'px-6 py-3 text-base',
      xl: 'px-8 py-4 text-lg'
    };

    const LoadingSpinner = () => (
      <motion.div
        className="w-4 h-4 border-2 border-white/30 border-t-white rounded-full"
        animate={{ rotate: 360 }}
        transition={{
          duration: 1,
          repeat: Infinity,
          ease: 'linear'
        }}
      />
    );

    return (
      <motion.button
        ref={ref}
        className={clsx(
          baseClasses,
          variantClasses[variant],
          sizeClasses[size],
          className
        )}
        variants={buttonVariants}
        initial="rest"
        whileHover={!disabled && !loading ? "hover" : "rest"}
        whileTap={!disabled && !loading ? "tap" : "rest"}
        animate={loading ? "loading" : "rest"}
        disabled={disabled || loading}
        {...props}
      >
        {loading && (
          <motion.div
            initial={{ opacity: 0, scale: 0 }}
            animate={{ opacity: 1, scale: 1 }}
            transition={transitions.smooth}
            className="mr-2"
          >
            <LoadingSpinner />
          </motion.div>
        )}
        
        {!loading && leftIcon && (
          <motion.div
            initial={{ opacity: 0, x: -10 }}
            animate={{ opacity: 1, x: 0 }}
            transition={transitions.gentle}
            className="mr-2 -ml-1"
          >
            {leftIcon}
          </motion.div>
        )}
        
        <motion.span
          initial={{ opacity: loading ? 0 : 1 }}
          animate={{ opacity: loading ? 0.7 : 1 }}
          transition={transitions.smooth}
        >
          {children}
        </motion.span>
        
        {!loading && rightIcon && (
          <motion.div
            initial={{ opacity: 0, x: 10 }}
            animate={{ opacity: 1, x: 0 }}
            transition={transitions.gentle}
            className="ml-2 -mr-1"
          >
            {rightIcon}
          </motion.div>
        )}
      </motion.button>
    );
  }
);

Button.displayName = 'Button';

export default Button;