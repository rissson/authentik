# Generated by Django 4.2.3 on 2023-07-29 12:41

import authentik.lib.generators
import authentik.lib.utils.time
import authentik.providers.kerberos.models
from django.conf import settings
import django.contrib.postgres.fields
import django.core.validators
from django.db import migrations, models
import django.db.models.deletion


class Migration(migrations.Migration):
    initial = True

    dependencies = [
        ("authentik_core", "0031_alter_user_type"),
        ("authentik_flows", "0025_alter_flowstagebinding_evaluate_on_plan_and_more"),
        migrations.swappable_dependency(settings.AUTH_USER_MODEL),
    ]

    operations = [
        migrations.CreateModel(
            name="KerberosKeys",
            fields=[
                (
                    "id",
                    models.AutoField(
                        auto_created=True, primary_key=True, serialize=False, verbose_name="ID"
                    ),
                ),
                (
                    "salt",
                    models.TextField(
                        default=authentik.lib.generators.generate_id,
                        help_text="Salt used to derive Kerberos keys.",
                    ),
                ),
                ("keys_raw", models.JSONField(default=dict, help_text="Kerberos keys stored raw.")),
                ("kvno", models.PositiveBigIntegerField(default=0, help_text="Version number.")),
            ],
        ),
        migrations.CreateModel(
            name="KerberosRealm",
            fields=[
                (
                    "id",
                    models.AutoField(
                        auto_created=True, primary_key=True, serialize=False, verbose_name="ID"
                    ),
                ),
                (
                    "maximum_ticket_lifetime",
                    models.TextField(
                        default="days=1",
                        help_text="Maximum Ticket lifetime (Format: hours=1;minutes=2;seconds=3).",
                        validators=[authentik.lib.utils.time.timedelta_string_validator],
                    ),
                ),
                (
                    "maximum_ticket_renew_lifetime",
                    models.TextField(
                        default="weeks=1",
                        help_text="Maximum Ticket renew lifetime (Format: hours=1;minutes=2;seconds=3).",
                        validators=[authentik.lib.utils.time.timedelta_string_validator],
                    ),
                ),
                (
                    "allowed_enctypes",
                    django.contrib.postgres.fields.ArrayField(
                        base_field=models.IntegerField(
                            choices=[
                                (17, "aes128-cts-hmac-sha1-96"),
                                (18, "aes256-cts-hmac-sha1-96"),
                                (19, "aes128-cts-hmac-sha256-128"),
                                (20, "aes256-cts-hmac-sha384-192"),
                            ]
                        ),
                        default=authentik.providers.kerberos.models._get_default_enctypes,
                        help_text="Allowed encryption types.",
                        size=None,
                    ),
                ),
                (
                    "allow_postdateable",
                    models.BooleanField(
                        default=True,
                        help_text="Should the user be able to request a ticket with a start time in the future.",
                    ),
                ),
                (
                    "allow_renewable",
                    models.BooleanField(
                        default=True,
                        help_text="Should the user getting the ticket be able to renew it.",
                    ),
                ),
                (
                    "allow_proxiable",
                    models.BooleanField(
                        default=True,
                        help_text="Should the service getting the ticket be able to use it on behalf of the user.",
                    ),
                ),
                (
                    "allow_forwardable",
                    models.BooleanField(
                        default=False,
                        help_text="Should the service getting the ticket be able to request a TGT on behalf of the user.",
                    ),
                ),
                (
                    "requires_preauth",
                    models.BooleanField(
                        default=True,
                        help_text="Should tickets only be issued to preauthenticated clients.",
                    ),
                ),
                (
                    "secret",
                    models.TextField(
                        default=authentik.lib.generators.generate_key,
                        help_text="The secret value from which Kerberos keys are derived.",
                    ),
                ),
                ("name", models.TextField(help_text="Kerberos realm name.", unique=True)),
                (
                    "maximum_skew",
                    models.TextField(
                        default="minutes=5",
                        help_text="Maximum allowed clock drift between the client and the server (Format: hours=1;minutes=2;seconds=3).",
                        validators=[authentik.lib.utils.time.timedelta_string_validator],
                    ),
                ),
                (
                    "authentication_flow",
                    models.ForeignKey(
                        help_text="Flow used for authentication when a TGT for the associated realm is requested by a user.",
                        null=True,
                        on_delete=django.db.models.deletion.SET_NULL,
                        to="authentik_flows.flow",
                    ),
                ),
            ],
            options={
                "verbose_name": "Kerberos Realm",
                "verbose_name_plural": "Kerberos Realms",
            },
        ),
        migrations.CreateModel(
            name="KerberosProvider",
            fields=[
                (
                    "provider_ptr",
                    models.OneToOneField(
                        auto_created=True,
                        on_delete=django.db.models.deletion.CASCADE,
                        parent_link=True,
                        primary_key=True,
                        serialize=False,
                        to="authentik_core.provider",
                    ),
                ),
                (
                    "maximum_ticket_lifetime",
                    models.TextField(
                        default="days=1",
                        help_text="Maximum Ticket lifetime (Format: hours=1;minutes=2;seconds=3).",
                        validators=[authentik.lib.utils.time.timedelta_string_validator],
                    ),
                ),
                (
                    "maximum_ticket_renew_lifetime",
                    models.TextField(
                        default="weeks=1",
                        help_text="Maximum Ticket renew lifetime (Format: hours=1;minutes=2;seconds=3).",
                        validators=[authentik.lib.utils.time.timedelta_string_validator],
                    ),
                ),
                (
                    "allowed_enctypes",
                    django.contrib.postgres.fields.ArrayField(
                        base_field=models.IntegerField(
                            choices=[
                                (17, "aes128-cts-hmac-sha1-96"),
                                (18, "aes256-cts-hmac-sha1-96"),
                                (19, "aes128-cts-hmac-sha256-128"),
                                (20, "aes256-cts-hmac-sha384-192"),
                            ]
                        ),
                        default=authentik.providers.kerberos.models._get_default_enctypes,
                        help_text="Allowed encryption types.",
                        size=None,
                    ),
                ),
                (
                    "allow_postdateable",
                    models.BooleanField(
                        default=True,
                        help_text="Should the user be able to request a ticket with a start time in the future.",
                    ),
                ),
                (
                    "allow_renewable",
                    models.BooleanField(
                        default=True,
                        help_text="Should the user getting the ticket be able to renew it.",
                    ),
                ),
                (
                    "allow_proxiable",
                    models.BooleanField(
                        default=True,
                        help_text="Should the service getting the ticket be able to use it on behalf of the user.",
                    ),
                ),
                (
                    "allow_forwardable",
                    models.BooleanField(
                        default=False,
                        help_text="Should the service getting the ticket be able to request a TGT on behalf of the user.",
                    ),
                ),
                (
                    "requires_preauth",
                    models.BooleanField(
                        default=True,
                        help_text="Should tickets only be issued to preauthenticated clients.",
                    ),
                ),
                (
                    "secret",
                    models.TextField(
                        default=authentik.lib.generators.generate_key,
                        help_text="The secret value from which Kerberos keys are derived.",
                    ),
                ),
                (
                    "service_principal_name",
                    models.TextField(
                        help_text="The Kerberos principal used to designate this provider, without the realm.",
                        validators=[
                            django.core.validators.RegexValidator(
                                "@.+$",
                                inverse_match=True,
                                message="SPN should not contain a realm.",
                            )
                        ],
                    ),
                ),
                (
                    "set_ok_as_delegate",
                    models.BooleanField(
                        default=False,
                        help_text="Should the tickets issued for this provider have the ok-as-delegate flag set.",
                    ),
                ),
                (
                    "realm",
                    models.ForeignKey(
                        on_delete=django.db.models.deletion.CASCADE,
                        to="authentik_providers_kerberos.kerberosrealm",
                    ),
                ),
            ],
            options={
                "verbose_name": "Kerberos Provider",
                "verbose_name_plural": "Kerberos Providers",
                "unique_together": {("realm", "service_principal_name")},
            },
            bases=("authentik_core.provider", models.Model),
        ),
        migrations.CreateModel(
            name="KerberosUserKeys",
            fields=[
                (
                    "kerberoskeys_ptr",
                    models.OneToOneField(
                        auto_created=True,
                        on_delete=django.db.models.deletion.CASCADE,
                        parent_link=True,
                        primary_key=True,
                        serialize=False,
                        to="authentik_providers_kerberos.kerberoskeys",
                    ),
                ),
                (
                    "target",
                    models.OneToOneField(
                        on_delete=django.db.models.deletion.CASCADE,
                        related_name="kerberoskeys",
                        to=settings.AUTH_USER_MODEL,
                    ),
                ),
            ],
            bases=("authentik_providers_kerberos.kerberoskeys",),
        ),
        migrations.CreateModel(
            name="KerberosRealmKeys",
            fields=[
                (
                    "kerberoskeys_ptr",
                    models.OneToOneField(
                        auto_created=True,
                        on_delete=django.db.models.deletion.CASCADE,
                        parent_link=True,
                        primary_key=True,
                        serialize=False,
                        to="authentik_providers_kerberos.kerberoskeys",
                    ),
                ),
                (
                    "target",
                    models.OneToOneField(
                        on_delete=django.db.models.deletion.CASCADE,
                        related_name="kerberoskeys",
                        to="authentik_providers_kerberos.kerberosrealm",
                    ),
                ),
            ],
            bases=("authentik_providers_kerberos.kerberoskeys",),
        ),
        migrations.CreateModel(
            name="KerberosProviderKeys",
            fields=[
                (
                    "kerberoskeys_ptr",
                    models.OneToOneField(
                        auto_created=True,
                        on_delete=django.db.models.deletion.CASCADE,
                        parent_link=True,
                        primary_key=True,
                        serialize=False,
                        to="authentik_providers_kerberos.kerberoskeys",
                    ),
                ),
                (
                    "target",
                    models.OneToOneField(
                        on_delete=django.db.models.deletion.CASCADE,
                        related_name="kerberoskeys",
                        to="authentik_providers_kerberos.kerberosprovider",
                    ),
                ),
            ],
            bases=("authentik_providers_kerberos.kerberoskeys",),
        ),
    ]
