(function loadDeviconCSS() {
    const link = document.createElement('link');
    link.rel = 'stylesheet';
    link.href = 'https://cdn.jsdelivr.net/gh/devicons/devicon@v2.16.0/devicon.min.css';
    document.head.appendChild(link);
})();

document.addEventListener('DOMContentLoaded', function() {
    initLangTabs();
    
    // Listen for theme changes to re-style tabs
    const observer = new MutationObserver(function(mutations) {
        mutations.forEach(function(mutation) {
            if (mutation.attributeName === 'class' && mutation.target.nodeName === 'HTML') {
                setTimeout(initLangTabs, 50);
            }
        });
    });
    
    observer.observe(document.documentElement, {
        attributes: true
    });
    
    // Also handle theme changes when page hash changes (mdbook sometimes updates theme this way)
    window.addEventListener('hashchange', function() {
        setTimeout(initLangTabs, 100);
    });
    
    // Handle page navigation
    window.addEventListener('load', function() {
        setTimeout(initLangTabs, 100);
    });
});

function initLangTabs() {
    const langTabsContainers = document.querySelectorAll('.langtabs');
    
    langTabsContainers.forEach(function(container) {
        const tabButtons = container.querySelectorAll('.langtabs-tab');
        
        tabButtons.forEach(function(button) {
            button.removeEventListener('click', handleTabClick);
            button.addEventListener('click', handleTabClick);
        });
        
        // Restore user preference localStorage
        try {
            const preferredLang = localStorage.getItem('mdbook-langtabs-preference');
            if (preferredLang) {
                const preferredButton = container.querySelector(`.langtabs-tab[data-lang="${preferredLang}"]`);
                if (preferredButton) {
                    preferredButton.click();
                    return;
                }
            }
        } catch (e) {
            // Ignore storage errors
        }
        
        // If no tab active select first
        if (!container.querySelector('.langtabs-tab.active')) {
            const firstButton = tabButtons[0];
            if (firstButton) {
                firstButton.click();
            }
        }
    });
}

function handleTabClick() {
    const container = this.closest('.langtabs');
    const lang = this.getAttribute('data-lang');
    
    // Deactivate all tabs in this container
    const tabButtons = container.querySelectorAll('.langtabs-tab');
    const tabContents = container.querySelectorAll('.langtabs-code');
    
    tabButtons.forEach(function(btn) {
        btn.classList.remove('active');
    });
    tabContents.forEach(function(content) {
        content.classList.remove('active');
    });
    
    // Activate selected tab
    this.classList.add('active');
    const activeContent = container.querySelector(`.langtabs-code[data-lang="${lang}"]`);
    if (activeContent) {
        activeContent.classList.add('active');
    }
    
    // Save preference to localStorage
    try {
        localStorage.setItem('mdbook-langtabs-preference', lang);
    } catch (e) {
        // Ignore storage errors
    }
}