import type { CheckoutField, CheckoutForm, FieldErrors } from './types';

/** 全角英数字・全角ハイフン類を半角に寄せる */
export function toHalfWidth(value: string): string {
  return value
    .replace(/[！-～]/g, (c) => String.fromCharCode(c.charCodeAt(0) - 0xfee0))
    .replace(/[‐－―ー−]/g, '-');
}

const EMAIL_RE = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;

export function validateField(field: CheckoutField, raw: string): string | undefined {
  const value = raw.trim();
  switch (field) {
    case 'name':
      if (!value) return 'お名前を入力してください';
      if (value.length > 50) return 'お名前は 50 文字以内で入力してください';
      return undefined;
    case 'email':
      if (!value) return 'メールアドレスを入力してください';
      if (!EMAIL_RE.test(toHalfWidth(value))) return 'メールアドレスの形式が正しくありません';
      return undefined;
    case 'postalCode':
      if (!value) return '郵便番号を入力してください';
      if (!/^\d{3}-?\d{4}$/.test(toHalfWidth(value)))
        return '郵便番号は 7 桁の数字で入力してください（例: 123-4567）';
      return undefined;
    case 'address':
      if (!value) return 'ご住所を入力してください';
      if (value.length > 200) return 'ご住所は 200 文字以内で入力してください';
      return undefined;
    case 'phone': {
      if (!value) return '電話番号を入力してください';
      const digits = toHalfWidth(value).replace(/-/g, '');
      if (!/^\d{10,11}$/.test(digits)) return '電話番号は 10〜11 桁の数字で入力してください';
      return undefined;
    }
    case 'paymentMethod':
      if (value !== 'card' && value !== 'cod') return 'お支払い方法を選択してください';
      return undefined;
  }
}

export const CHECKOUT_FIELDS: CheckoutField[] = [
  'name',
  'email',
  'postalCode',
  'address',
  'phone',
  'paymentMethod',
];

export function validateCheckout(form: Partial<CheckoutForm>): FieldErrors {
  const errors: FieldErrors = {};
  for (const field of CHECKOUT_FIELDS) {
    const value = form[field];
    const message = validateField(field, typeof value === 'string' ? value : '');
    if (message) errors[field] = message;
  }
  return errors;
}

/** 検証済みフォームを保存用に正規化する */
export function normalizeCheckout(form: CheckoutForm): CheckoutForm {
  return {
    name: form.name.trim(),
    email: toHalfWidth(form.email.trim()),
    postalCode: toHalfWidth(form.postalCode.trim()),
    address: form.address.trim(),
    phone: toHalfWidth(form.phone.trim()),
    paymentMethod: form.paymentMethod,
  };
}
