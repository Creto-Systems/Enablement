import { Client, Contact, BillingInfo, ContractTerms, Address } from '@shared/types';
import { v4 as uuidv4 } from 'uuid';

export class ClientModel {
  static create(data: Partial<Client>): Client {
    const now = new Date();

    return {
      id: uuidv4(),
      name: data.name || '',
      industry: data.industry || '',
      parentClientId: data.parentClientId,
      contacts: data.contacts || [],
      billingInfo: data.billingInfo || this.getDefaultBillingInfo(),
      contractTerms: data.contractTerms || this.getDefaultContractTerms(),
      metadata: {
        createdAt: now,
        updatedAt: now,
        status: data.metadata?.status || 'Prospect',
      },
    };
  }

  static update(client: Client, updates: Partial<Client>): Client {
    return {
      ...client,
      ...updates,
      metadata: {
        ...client.metadata,
        updatedAt: new Date(),
      },
    };
  }

  static addContact(client: Client, contact: Omit<Contact, 'id'>): Client {
    const newContact: Contact = {
      id: uuidv4(),
      ...contact,
    };

    return {
      ...client,
      contacts: [...client.contacts, newContact],
      metadata: {
        ...client.metadata,
        updatedAt: new Date(),
      },
    };
  }

  static updateBillingInfo(client: Client, billingInfo: Partial<BillingInfo>): Client {
    return {
      ...client,
      billingInfo: {
        ...client.billingInfo,
        ...billingInfo,
      },
      metadata: {
        ...client.metadata,
        updatedAt: new Date(),
      },
    };
  }

  static updateContractTerms(client: Client, contractTerms: Partial<ContractTerms>): Client {
    return {
      ...client,
      contractTerms: {
        ...client.contractTerms,
        ...contractTerms,
      },
      metadata: {
        ...client.metadata,
        updatedAt: new Date(),
      },
    };
  }

  static deactivate(client: Client): Client {
    return {
      ...client,
      metadata: {
        ...client.metadata,
        status: 'Inactive',
        updatedAt: new Date(),
      },
    };
  }

  static activate(client: Client): Client {
    return {
      ...client,
      metadata: {
        ...client.metadata,
        status: 'Active',
        updatedAt: new Date(),
      },
    };
  }

  static getPrimaryContact(client: Client): Contact | undefined {
    return client.contacts.find((c) => c.isPrimary);
  }

  static isActive(client: Client): boolean {
    return client.metadata.status === 'Active';
  }

  static hasActiveMSA(client: Client): boolean {
    if (!client.contractTerms.msa) return false;
    if (!client.contractTerms.msaExpiryDate) return true;
    return new Date(client.contractTerms.msaExpiryDate) > new Date();
  }

  private static getDefaultBillingInfo(): BillingInfo {
    return {
      paymentTerms: 30,
      preferredMethod: 'ACH',
      taxId: '',
      currency: 'USD',
      billingAddress: {
        street: '',
        city: '',
        state: '',
        postalCode: '',
        country: 'US',
      },
    };
  }

  private static getDefaultContractTerms(): ContractTerms {
    return {
      defaultRate: 0,
      discountTier: 0,
      msa: false,
    };
  }

  static validate(client: Partial<Client>): { isValid: boolean; errors: string[] } {
    const errors: string[] = [];

    if (!client.name || client.name.trim() === '') {
      errors.push('Client name is required');
    }

    if (!client.industry || client.industry.trim() === '') {
      errors.push('Industry is required');
    }

    if (client.billingInfo) {
      if (!client.billingInfo.taxId || client.billingInfo.taxId.trim() === '') {
        errors.push('Tax ID is required');
      }

      if (client.billingInfo.paymentTerms < 0) {
        errors.push('Payment terms must be positive');
      }
    }

    if (client.contractTerms) {
      if (client.contractTerms.defaultRate < 0) {
        errors.push('Default rate must be positive');
      }

      if (client.contractTerms.discountTier < 0 || client.contractTerms.discountTier > 100) {
        errors.push('Discount tier must be between 0 and 100');
      }
    }

    return {
      isValid: errors.length === 0,
      errors,
    };
  }
}
