import React, { useState } from 'react';
import type { CreateTripRequest, ActivityType } from '../../shared/types';

interface TripFormProps {
  onSubmit: (request: CreateTripRequest) => Promise<void>;
  loading: boolean;
}

const TripForm: React.FC<TripFormProps> = ({ onSubmit, loading }) => {
  const [formData, setFormData] = useState<CreateTripRequest>({
    destination: '',
    startDate: '',
    endDate: '',
    budget: {
      min: 1000,
      max: 5000,
      currency: 'USD',
    },
    travelerCount: 1,
    preferences: {
      activityTypes: [],
      accommodationType: 'hotel',
      flightClass: 'economy',
      pace: 'moderate',
    },
  });

  const [errors, setErrors] = useState<Record<string, string>>({});

  const activityTypes: { value: ActivityType; label: string }[] = [
    { value: 'adventure', label: 'Adventure' },
    { value: 'culture', label: 'Culture' },
    { value: 'relaxation', label: 'Relaxation' },
    { value: 'food', label: 'Food & Dining' },
    { value: 'shopping', label: 'Shopping' },
    { value: 'nature', label: 'Nature' },
  ];

  const validate = (): boolean => {
    const newErrors: Record<string, string> = {};

    if (!formData.destination.trim()) {
      newErrors.destination = 'Destination is required';
    }
    if (!formData.startDate) {
      newErrors.startDate = 'Start date is required';
    }
    if (!formData.endDate) {
      newErrors.endDate = 'End date is required';
    }
    if (formData.startDate && formData.endDate && formData.startDate >= formData.endDate) {
      newErrors.endDate = 'End date must be after start date';
    }
    if (formData.budget.min <= 0) {
      newErrors.budgetMin = 'Minimum budget must be positive';
    }
    if (formData.budget.max < formData.budget.min) {
      newErrors.budgetMax = 'Maximum budget must be greater than minimum';
    }
    if (formData.travelerCount < 1) {
      newErrors.travelerCount = 'At least one traveler required';
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (validate()) {
      await onSubmit(formData);
    }
  };

  const toggleActivityType = (type: ActivityType) => {
    const current = formData.preferences.activityTypes;
    const updated = current.includes(type)
      ? current.filter((t) => t !== type)
      : [...current, type];

    setFormData({
      ...formData,
      preferences: { ...formData.preferences, activityTypes: updated },
    });
  };

  return (
    <form onSubmit={handleSubmit} className="space-y-6">
      {/* Destination */}
      <div>
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          Destination
        </label>
        <input
          type="text"
          value={formData.destination}
          onChange={(e) => setFormData({ ...formData, destination: e.target.value })}
          placeholder="e.g., Paris, France"
          className="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-primary focus:border-transparent dark:bg-gray-700 dark:text-white"
        />
        {errors.destination && (
          <p className="mt-1 text-sm text-red-600 dark:text-red-400">{errors.destination}</p>
        )}
      </div>

      {/* Dates */}
      <div className="grid grid-cols-2 gap-4">
        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            Start Date
          </label>
          <input
            type="date"
            value={formData.startDate}
            onChange={(e) => setFormData({ ...formData, startDate: e.target.value })}
            min={new Date().toISOString().split('T')[0]}
            className="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-primary focus:border-transparent dark:bg-gray-700 dark:text-white"
          />
          {errors.startDate && (
            <p className="mt-1 text-sm text-red-600 dark:text-red-400">{errors.startDate}</p>
          )}
        </div>
        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            End Date
          </label>
          <input
            type="date"
            value={formData.endDate}
            onChange={(e) => setFormData({ ...formData, endDate: e.target.value })}
            min={formData.startDate || new Date().toISOString().split('T')[0]}
            className="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-primary focus:border-transparent dark:bg-gray-700 dark:text-white"
          />
          {errors.endDate && (
            <p className="mt-1 text-sm text-red-600 dark:text-red-400">{errors.endDate}</p>
          )}
        </div>
      </div>

      {/* Budget */}
      <div>
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          Budget Range ({formData.budget.currency})
        </label>
        <div className="grid grid-cols-2 gap-4">
          <div>
            <input
              type="number"
              value={formData.budget.min}
              onChange={(e) =>
                setFormData({
                  ...formData,
                  budget: { ...formData.budget, min: Number(e.target.value) },
                })
              }
              placeholder="Minimum"
              className="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-primary focus:border-transparent dark:bg-gray-700 dark:text-white"
            />
            {errors.budgetMin && (
              <p className="mt-1 text-sm text-red-600 dark:text-red-400">{errors.budgetMin}</p>
            )}
          </div>
          <div>
            <input
              type="number"
              value={formData.budget.max}
              onChange={(e) =>
                setFormData({
                  ...formData,
                  budget: { ...formData.budget, max: Number(e.target.value) },
                })
              }
              placeholder="Maximum"
              className="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-primary focus:border-transparent dark:bg-gray-700 dark:text-white"
            />
            {errors.budgetMax && (
              <p className="mt-1 text-sm text-red-600 dark:text-red-400">{errors.budgetMax}</p>
            )}
          </div>
        </div>
      </div>

      {/* Travelers */}
      <div>
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          Number of Travelers
        </label>
        <input
          type="number"
          min="1"
          max="20"
          value={formData.travelerCount}
          onChange={(e) => setFormData({ ...formData, travelerCount: Number(e.target.value) })}
          className="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-primary focus:border-transparent dark:bg-gray-700 dark:text-white"
        />
        {errors.travelerCount && (
          <p className="mt-1 text-sm text-red-600 dark:text-red-400">{errors.travelerCount}</p>
        )}
      </div>

      {/* Activity Preferences */}
      <div>
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          Activity Preferences
        </label>
        <div className="flex flex-wrap gap-2">
          {activityTypes.map((activity) => (
            <button
              key={activity.value}
              type="button"
              onClick={() => toggleActivityType(activity.value)}
              className={`px-4 py-2 rounded-full text-sm font-medium transition-colors ${
                formData.preferences.activityTypes.includes(activity.value)
                  ? 'bg-primary text-white'
                  : 'bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300 hover:bg-gray-300 dark:hover:bg-gray-600'
              }`}
            >
              {activity.label}
            </button>
          ))}
        </div>
      </div>

      {/* Other Preferences */}
      <div className="grid grid-cols-3 gap-4">
        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            Accommodation
          </label>
          <select
            value={formData.preferences.accommodationType}
            onChange={(e) =>
              setFormData({
                ...formData,
                preferences: {
                  ...formData.preferences,
                  accommodationType: e.target.value as any,
                },
              })
            }
            className="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-primary focus:border-transparent dark:bg-gray-700 dark:text-white"
          >
            <option value="hotel">Hotel</option>
            <option value="hostel">Hostel</option>
            <option value="airbnb">Airbnb</option>
            <option value="resort">Resort</option>
          </select>
        </div>
        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            Flight Class
          </label>
          <select
            value={formData.preferences.flightClass}
            onChange={(e) =>
              setFormData({
                ...formData,
                preferences: { ...formData.preferences, flightClass: e.target.value as any },
              })
            }
            className="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-primary focus:border-transparent dark:bg-gray-700 dark:text-white"
          >
            <option value="economy">Economy</option>
            <option value="premium">Premium Economy</option>
            <option value="business">Business</option>
            <option value="first">First Class</option>
          </select>
        </div>
        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            Trip Pace
          </label>
          <select
            value={formData.preferences.pace}
            onChange={(e) =>
              setFormData({
                ...formData,
                preferences: { ...formData.preferences, pace: e.target.value as any },
              })
            }
            className="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg focus:ring-2 focus:ring-primary focus:border-transparent dark:bg-gray-700 dark:text-white"
          >
            <option value="relaxed">Relaxed</option>
            <option value="moderate">Moderate</option>
            <option value="packed">Packed</option>
          </select>
        </div>
      </div>

      {/* Submit Button */}
      <button
        type="submit"
        disabled={loading}
        className="w-full bg-primary text-white py-3 px-6 rounded-lg font-medium hover:bg-primary/90 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
      >
        {loading ? 'Planning Your Trip...' : 'Start Planning'}
      </button>
    </form>
  );
};

export default TripForm;
