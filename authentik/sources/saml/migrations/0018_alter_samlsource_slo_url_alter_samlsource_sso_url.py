# Generated by Django 5.0.13 on 2025-03-31 13:53

import authentik.lib.models
from django.db import migrations, models


class Migration(migrations.Migration):

    dependencies = [
        ("authentik_sources_saml", "0017_fix_x509subjectname"),
    ]

    operations = [
        migrations.AlterField(
            model_name="samlsource",
            name="slo_url",
            field=models.TextField(
                blank=True,
                default=None,
                help_text="Optional URL if your IDP supports Single-Logout.",
                null=True,
                validators=[authentik.lib.models.DomainlessURLValidator(schemes=("http", "https"))],
                verbose_name="SLO URL",
            ),
        ),
        migrations.AlterField(
            model_name="samlsource",
            name="sso_url",
            field=models.TextField(
                help_text="URL that the initial Login request is sent to.",
                validators=[authentik.lib.models.DomainlessURLValidator(schemes=("http", "https"))],
                verbose_name="SSO URL",
            ),
        ),
    ]
