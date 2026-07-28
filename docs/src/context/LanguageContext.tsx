import { createContext, useContext, useEffect, useState, type ReactNode } from 'react';
import { useTranslation } from 'react-i18next';

type Language = 'en' | 'pt-BR' | 'pt-PT';

interface LanguageContextType {
  language: Language;
  setLanguage: (lang: Language) => void;
  t: (key: string) => string;
}

const LanguageContext = createContext<LanguageContextType | undefined>(undefined);

export function LanguageProvider({ children }: { children: ReactNode }) {
  const { i18n } = useTranslation();
  const [language, setLanguageState] = useState<Language>(() => {
    const saved = localStorage.getItem('i18nextLng');
    if (saved && ['en', 'pt-BR', 'pt-PT'].includes(saved)) {
      return saved as Language;
    }
    return 'en';
  });

  useEffect(() => {
    i18n.changeLanguage(language);
    localStorage.setItem('i18nextLng', language);
  }, [language, i18n]);

  const setLanguage = (lang: Language) => {
    setLanguageState(lang);
  };

  return (
    <LanguageContext.Provider value={{ language, setLanguage, t: i18n.t.bind(i18n) }}>
      {children}
    </LanguageContext.Provider>
  );
}

export function useLanguage() {
  const context = useContext(LanguageContext);
  if (!context) {
    throw new Error('useLanguage must be used within a LanguageProvider');
  }
  return context;
}