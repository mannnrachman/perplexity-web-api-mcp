document.addEventListener('DOMContentLoaded', () => {
    // Tab switching
    const tabButtons = document.querySelectorAll('.tab-btn');
    const tabContents = document.querySelectorAll('.tab-content');

    tabButtons.forEach(button => {
        button.addEventListener('click', () => {
            const tabId = button.dataset.tab;
            
            tabButtons.forEach(btn => btn.classList.remove('active'));
            tabContents.forEach(content => content.classList.remove('active'));
            
            button.classList.add('active');
            document.getElementById(tabId).classList.add('active');
        });
    });

    // Copying functionality
    const copyBtn = document.querySelector('.copy-btn');
    
    copyBtn.addEventListener('click', async () => {
        const activeCode = document.querySelector('.tab-content.active code');
        if (!activeCode) return;
        
        // Get text without syntax highlighting spans
        const text = activeCode.textContent.replace(/\\\n\s*/g, ' ');
        
        try {
            await navigator.clipboard.writeText(text);
            
            const icon = copyBtn.querySelector('i');
            icon.classList.replace('fa-regular', 'fa-solid');
            icon.classList.replace('fa-copy', 'fa-check');
            copyBtn.style.color = 'var(--color-primary)';
            
            setTimeout(() => {
                icon.classList.replace('fa-solid', 'fa-regular');
                icon.classList.replace('fa-check', 'fa-copy');
                copyBtn.style.color = '';
            }, 2000);
        } catch (err) {
            console.error('Failed to copy:', err);
        }
    });
});
