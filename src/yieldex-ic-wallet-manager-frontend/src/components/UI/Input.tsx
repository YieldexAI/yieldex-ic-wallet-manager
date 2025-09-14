import React from 'react';
import { motion } from 'framer-motion';
import { transitions } from '@/utils/animations';
import { clsx } from 'clsx';

export interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  error?: string;
  helperText?: string;
  leftIcon?: React.ReactNode;
  rightIcon?: React.ReactNode;
  rightElement?: React.ReactNode;
  variant?: 'default' | 'ghost' | 'filled';
  inputSize?: 'sm' | 'md' | 'lg';
}

const Input = React.forwardRef<HTMLInputElement, InputProps>(
  ({
    className,
    label,
    error,
    helperText,
    leftIcon,
    rightIcon,
    rightElement,
    variant = 'default',
    inputSize = 'md',
    disabled,
    ...props
  }, ref) => {
    
    const baseClasses = clsx(
      'w-full rounded-lg border font-medium transition-all duration-200',
      'focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-gray-900',
      'disabled:opacity-50 disabled:cursor-not-allowed',
      'placeholder:text-gray-500'
    );

    const variantClasses = {
      default: clsx(
        'bg-gray-800 border-gray-600 text-white',
        'hover:border-gray-500 focus:border-primary-500 focus:ring-primary-500',
        error && 'border-red-500 focus:border-red-500 focus:ring-red-500'
      ),
      ghost: clsx(
        'bg-transparent border-gray-600 text-white',
        'hover:bg-gray-800/30 hover:border-gray-500',
        'focus:bg-gray-800/50 focus:border-primary-500 focus:ring-primary-500',
        error && 'border-red-500 focus:border-red-500 focus:ring-red-500'
      ),
      filled: clsx(
        'bg-gray-700 border-transparent text-white',
        'hover:bg-gray-600 focus:bg-gray-800 focus:border-primary-500 focus:ring-primary-500',
        error && 'bg-red-900/20 border-red-500 focus:border-red-500 focus:ring-red-500'
      )
    };

    const sizeClasses = {
      sm: 'px-3 py-2 text-sm',
      md: 'px-4 py-2.5 text-sm',
      lg: 'px-4 py-3 text-base'
    };

    const iconSizeClasses = {
      sm: 'w-4 h-4',
      md: 'w-5 h-5',
      lg: 'w-5 h-5'
    };

    return (
      <div className="w-full">
        {/* Label */}
        {label && (
          <motion.label
            className="block text-sm font-medium text-gray-300 mb-2"
            initial={{ opacity: 0, y: -10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={transitions.gentle}
          >
            {label}
          </motion.label>
        )}

        {/* Input Container */}
        <div className="relative">
          {/* Left Icon */}
          {leftIcon && (
            <div className="absolute inset-y-0 left-0 flex items-center pl-3 pointer-events-none">
              <div className={clsx('text-gray-400', iconSizeClasses[inputSize])}>
                {leftIcon}
              </div>
            </div>
          )}

          {/* Input */}
          <motion.div
            initial={{ opacity: 0, scale: 0.95 }}
            animate={{ opacity: 1, scale: 1 }}
            transition={transitions.gentle}
          >
            <input
              ref={ref}
              className={clsx(
                baseClasses,
                variantClasses[variant],
                sizeClasses[inputSize],
                leftIcon && 'pl-10',
                (rightIcon || rightElement) && 'pr-10',
                className
              )}
              disabled={disabled}
              {...props}
            />
          </motion.div>

          {/* Right Icon */}
          {rightIcon && !rightElement && (
            <div className="absolute inset-y-0 right-0 flex items-center pr-3 pointer-events-none">
              <div className={clsx('text-gray-400', iconSizeClasses[inputSize])}>
                {rightIcon}
              </div>
            </div>
          )}

          {/* Right Element */}
          {rightElement && (
            <div className="absolute inset-y-0 right-0 flex items-center pr-3">
              {rightElement}
            </div>
          )}
        </div>

        {/* Helper Text or Error */}
        {(error || helperText) && (
          <motion.p
            className={clsx(
              'mt-2 text-sm',
              error ? 'text-red-400' : 'text-gray-400'
            )}
            initial={{ opacity: 0, y: -5 }}
            animate={{ opacity: 1, y: 0 }}
            transition={transitions.gentle}
          >
            {error || helperText}
          </motion.p>
        )}
      </div>
    );
  }
);

Input.displayName = 'Input';

// Number input with token selector
export const TokenAmountInput: React.FC<{
  value: string;
  onChange: (value: string) => void;
  selectedToken: string;
  onTokenChange?: (token: string) => void;
  availableTokens?: string[];
  balance?: number;
  error?: string;
  label?: string;
  placeholder?: string;
  disabled?: boolean;
  showMaxButton?: boolean;
}> = ({
  value,
  onChange,
  selectedToken,
  onTokenChange,
  availableTokens = ['USDC', 'USDT', 'DAI'],
  balance,
  error,
  label = 'Amount',
  placeholder = '0.0',
  disabled,
  showMaxButton = true
}) => {
  const handleMaxClick = () => {
    if (balance) {
      onChange(balance.toString());
    }
  };

  return (
    <div>
      {label && (
        <div className="flex items-center justify-between mb-2">
          <label className="block text-sm font-medium text-gray-300">
            {label}
          </label>
          {balance !== undefined && (
            <span className="text-sm text-gray-400">
              Balance: {balance.toFixed(2)} {selectedToken}
            </span>
          )}
        </div>
      )}

      <div className="relative">
        <Input
          type="number"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          placeholder={placeholder}
          disabled={disabled}
          error={error}
          className="pr-24"
          step="any"
          min="0"
        />

        <div className="absolute inset-y-0 right-0 flex items-center pr-3 space-x-2">
          {showMaxButton && balance && (
            <button
              type="button"
              onClick={handleMaxClick}
              className="text-xs text-primary-400 hover:text-primary-300 font-medium"
              disabled={disabled}
            >
              MAX
            </button>
          )}

          {onTokenChange ? (
            <select
              value={selectedToken}
              onChange={(e) => onTokenChange(e.target.value)}
              className="bg-transparent text-white text-sm font-medium border-none focus:outline-none cursor-pointer"
              disabled={disabled}
            >
              {availableTokens.map((token) => (
                <option key={token} value={token} className="bg-gray-800">
                  {token}
                </option>
              ))}
            </select>
          ) : (
            <span className="text-sm font-medium text-gray-300">
              {selectedToken}
            </span>
          )}
        </div>
      </div>
    </div>
  );
};

export default Input;