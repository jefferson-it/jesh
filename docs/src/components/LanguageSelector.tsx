import { useLanguage } from '../context/LanguageContext';

const languages = [
  { code: 'en', label: 'English', flag: '🇺🇸' },
  { code: 'pt-BR', label: 'Português (Brasil)', flag: '🇧🇷' },
  { code: 'pt-PT', label: 'Português (Portugal)', flag: '🇵🇹' },
] as const;

export function LanguageSelector() {
  const { language, setLanguage } = useLanguage();

  return (
    <div className="language-selector">
      <button className="lang-btn" aria-label="Select language">
        <span className="lang-flag">
          {languages.find(l => l.code === language)?.flag || '🌐'}
        </span>
        <span className="lang-label">
          {languages.find(l => l.code === language)?.label || 'Language'}
        </span>
        <svg className="lang-arrow" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <path d="M6 9l6 6 6-6" />
        </svg>
      </button>
      <ul className="lang-dropdown" role="listbox">
        {languages.map(({ code, label, flag }) => (
          <li key={code} role="option" aria-selected={language === code}>
            <button
              className={`lang-option ${language === code ? 'active' : ''}`}
              onClick={() => setLanguage(code as 'en' | 'pt-BR' | 'pt-PT')}
            >
              <span className="lang-flag">{flag}</span>
              <span className="lang-label">{label}</span>
              {language === code && <span className="lang-check">✓</span>}
            </button>
          </li>
        ))}
      </ul>
    </div>
  );
}