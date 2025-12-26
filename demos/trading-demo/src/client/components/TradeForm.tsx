import React, { useState, useEffect } from 'react';
import clsx from 'clsx';

export interface TradeFormData {
  agentId: string;
  symbol: string;
  side: 'buy' | 'sell';
  quantity: number;
  price: number;
  reasoning: string;
}

export interface TradeFormProps {
  agentId: string;
  onSubmit: (data: TradeFormData) => void | Promise<void>;
  loading?: boolean;
  maxTradeSize?: number;
}

interface FormErrors {
  symbol?: string;
  quantity?: string;
  price?: string;
  reasoning?: string;
}

export function TradeForm({
  agentId,
  onSubmit,
  loading = false,
  maxTradeSize = 10000,
}: TradeFormProps) {
  const [symbol, setSymbol] = useState('');
  const [side, setSide] = useState<'buy' | 'sell'>('buy');
  const [quantity, setQuantity] = useState('');
  const [price, setPrice] = useState('');
  const [reasoning, setReasoning] = useState('');
  const [errors, setErrors] = useState<FormErrors>({});
  const [touched, setTouched] = useState<Set<string>>(new Set());

  const totalValue = parseFloat(quantity || '0') * parseFloat(price || '0');
  const isLargeTrade = totalValue > maxTradeSize * 0.5;

  useEffect(() => {
    // Reset form after successful submission
    if (!loading && Object.keys(errors).length === 0 && touched.size > 0) {
      const hasValues = symbol || quantity || price || reasoning;
      if (!hasValues) {
        setTouched(new Set());
      }
    }
  }, [loading, errors, symbol, quantity, price, reasoning, touched.size]);

  const validateField = (name: string, value: string): string | undefined => {
    switch (name) {
      case 'symbol':
        if (!value) return 'Symbol is required';
        if (!/^[A-Z]{1,5}$/.test(value)) return 'Invalid symbol format';
        return undefined;
      case 'quantity':
        if (!value || parseFloat(value) <= 0) {
          return 'Quantity must be greater than 0';
        }
        return undefined;
      case 'price':
        if (!value || parseFloat(value) <= 0) {
          return 'Price must be positive';
        }
        return undefined;
      case 'reasoning':
        if (!value) return 'Reasoning is required';
        if (value.length < 10) return 'Reasoning must be at least 10 characters';
        return undefined;
      default:
        return undefined;
    }
  };

  const handleBlur = (field: string) => {
    setTouched((prev) => new Set(prev).add(field));
    const value =
      field === 'symbol'
        ? symbol
        : field === 'quantity'
        ? quantity
        : field === 'price'
        ? price
        : reasoning;
    const error = validateField(field, value);
    setErrors((prev) => ({
      ...prev,
      [field]: error,
    }));
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    // Validate all fields
    const newErrors: FormErrors = {
      symbol: validateField('symbol', symbol),
      quantity: validateField('quantity', quantity),
      price: validateField('price', price),
      reasoning: validateField('reasoning', reasoning),
    };

    // Remove undefined errors
    Object.keys(newErrors).forEach((key) => {
      if (!newErrors[key as keyof FormErrors]) {
        delete newErrors[key as keyof FormErrors];
      }
    });

    setErrors(newErrors);
    setTouched(new Set(['symbol', 'quantity', 'price', 'reasoning']));

    if (Object.keys(newErrors).length === 0) {
      await onSubmit({
        agentId,
        symbol: symbol.toUpperCase(),
        side,
        quantity: parseFloat(quantity),
        price: parseFloat(price),
        reasoning,
      });

      // Reset form
      setSymbol('');
      setQuantity('');
      setPrice('');
      setReasoning('');
      setSide('buy');
      setTouched(new Set());
      setErrors({});
    }
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-4">
      {/* Symbol */}
      <div>
        <label htmlFor="symbol" className="block text-sm font-medium text-gray-700 mb-1">
          Symbol
        </label>
        <input
          id="symbol"
          type="text"
          value={symbol}
          onChange={(e) => setSymbol(e.target.value.toUpperCase())}
          onBlur={() => handleBlur('symbol')}
          className={clsx(
            'w-full px-3 py-2 border rounded-md focus:outline-none focus:ring-2',
            {
              'border-red-500 focus:ring-red-500': touched.has('symbol') && errors.symbol,
              'border-gray-300 focus:ring-blue-500': !touched.has('symbol') || !errors.symbol,
            }
          )}
          placeholder="AAPL"
          maxLength={5}
          disabled={loading}
        />
        {touched.has('symbol') && errors.symbol && (
          <p className="mt-1 text-sm text-red-600">{errors.symbol}</p>
        )}
      </div>

      {/* Side */}
      <div>
        <label htmlFor="side" className="block text-sm font-medium text-gray-700 mb-1">
          Side
        </label>
        <select
          id="side"
          value={side}
          onChange={(e) => setSide(e.target.value as 'buy' | 'sell')}
          className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
          disabled={loading}
        >
          <option value="buy">Buy</option>
          <option value="sell">Sell</option>
        </select>
      </div>

      {/* Quantity */}
      <div>
        <label htmlFor="quantity" className="block text-sm font-medium text-gray-700 mb-1">
          Quantity
        </label>
        <input
          id="quantity"
          type="number"
          value={quantity}
          onChange={(e) => setQuantity(e.target.value)}
          onBlur={() => handleBlur('quantity')}
          className={clsx(
            'w-full px-3 py-2 border rounded-md focus:outline-none focus:ring-2',
            {
              'border-red-500 focus:ring-red-500': touched.has('quantity') && errors.quantity,
              'border-gray-300 focus:ring-blue-500': !touched.has('quantity') || !errors.quantity,
            }
          )}
          placeholder="10"
          min="1"
          step="1"
          disabled={loading}
        />
        {touched.has('quantity') && errors.quantity && (
          <p className="mt-1 text-sm text-red-600">{errors.quantity}</p>
        )}
      </div>

      {/* Price */}
      <div>
        <label htmlFor="price" className="block text-sm font-medium text-gray-700 mb-1">
          Price
        </label>
        <input
          id="price"
          type="number"
          value={price}
          onChange={(e) => setPrice(e.target.value)}
          onBlur={() => handleBlur('price')}
          className={clsx(
            'w-full px-3 py-2 border rounded-md focus:outline-none focus:ring-2',
            {
              'border-red-500 focus:ring-red-500': touched.has('price') && errors.price,
              'border-gray-300 focus:ring-blue-500': !touched.has('price') || !errors.price,
            }
          )}
          placeholder="150.00"
          min="0.01"
          step="0.01"
          disabled={loading}
        />
        {touched.has('price') && errors.price && (
          <p className="mt-1 text-sm text-red-600">{errors.price}</p>
        )}
      </div>

      {/* Total Value */}
      {totalValue > 0 && (
        <div className="bg-gray-50 p-3 rounded-md">
          <span className="text-sm font-medium text-gray-700">
            Total: ${totalValue.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
          </span>
        </div>
      )}

      {/* Large Trade Warning */}
      {isLargeTrade && (
        <div className="bg-yellow-50 border border-yellow-200 p-3 rounded-md">
          <p className="text-sm font-medium text-yellow-800">Large Trade Warning</p>
          <p className="text-sm text-yellow-700">
            This trade exceeds 50% of maximum trade size
          </p>
        </div>
      )}

      {/* Reasoning */}
      <div>
        <label htmlFor="reasoning" className="block text-sm font-medium text-gray-700 mb-1">
          Reasoning
        </label>
        <textarea
          id="reasoning"
          value={reasoning}
          onChange={(e) => setReasoning(e.target.value)}
          onBlur={() => handleBlur('reasoning')}
          className={clsx(
            'w-full px-3 py-2 border rounded-md focus:outline-none focus:ring-2',
            {
              'border-red-500 focus:ring-red-500': touched.has('reasoning') && errors.reasoning,
              'border-gray-300 focus:ring-blue-500':
                !touched.has('reasoning') || !errors.reasoning,
            }
          )}
          placeholder="Explain the reasoning for this trade..."
          rows={3}
          disabled={loading}
        />
        {touched.has('reasoning') && errors.reasoning && (
          <p className="mt-1 text-sm text-red-600">{errors.reasoning}</p>
        )}
      </div>

      {/* Submit Button */}
      <button
        type="submit"
        disabled={loading}
        className={clsx(
          'w-full px-4 py-2 rounded-md font-medium transition-colors',
          'focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500',
          {
            'bg-blue-600 text-white hover:bg-blue-700': !loading,
            'bg-gray-400 text-gray-200 cursor-not-allowed': loading,
          }
        )}
      >
        {loading ? 'Submitting...' : 'Submit Trade'}
      </button>
    </form>
  );
}
